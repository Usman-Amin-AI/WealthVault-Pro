use crate::db::{get_connection, WriteHandle};
use crate::errors::Result;
use crate::goals::goals_model::{Goal, GoalsAllocation, NewGoal};
use crate::goals::goals_traits::GoalRepositoryTrait;
use crate::schema::goals;
use crate::schema::goals::dsl::{goals as goals_dsl, id};
use crate::schema::goals_allocation;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;

use std::sync::Arc;
use uuid::Uuid;

pub struct GoalRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl GoalRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        GoalRepository { pool, writer }
    }

    pub fn load_goals_impl(&self, profile_id: &str) -> Result<Vec<Goal>> {
        let mut conn = get_connection(&self.pool)?;
        Ok(goals::table
            .filter(goals::profile_id.eq(profile_id))
            .load::<Goal>(&mut conn)?)
    }

    pub fn load_allocations_for_non_achieved_goals_impl(&self, profile_id_param: &str) -> Result<Vec<GoalsAllocation>> {
        let mut conn = get_connection(&self.pool)?;
        Ok(goals_allocation::table
            .inner_join(goals::table.on(goals::id.eq(goals_allocation::goal_id)))
            .filter(goals::is_achieved.eq(false).and(goals::profile_id.eq(profile_id_param)))
            .select((
                goals_allocation::id,
                goals_allocation::goal_id,
                goals_allocation::account_id,
                goals_allocation::percent_allocation,
            ))
            .load::<GoalsAllocation>(&mut conn)?)
    }
}

#[async_trait]
impl GoalRepositoryTrait for GoalRepository {
    fn load_goals(&self, profile_id: &str) -> Result<Vec<Goal>> {
        self.load_goals_impl(profile_id)
    }

    async fn insert_new_goal(&self, new_goal: NewGoal, profile_id_param: &str) -> Result<Goal> {
        let profile_id_owned = profile_id_param.to_string();
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<Goal> {
                let mut new_goal_mut = new_goal;
                new_goal_mut.id = Some(Uuid::new_v4().to_string());
                if new_goal_mut.profile_id.is_none() {
                    new_goal_mut.profile_id = Some(profile_id_owned);
                }

                Ok(diesel::insert_into(goals::table)
                    .values(&new_goal_mut)
                    .returning(goals::all_columns)
                    .get_result(conn)?)
            })
            .await
    }

    async fn update_goal(&self, goal_update: Goal, profile_id_param: &str) -> Result<Goal> {
        let goal_id_owned = goal_update.id.clone();
        let mut goal_update_owned = goal_update.clone();
        goal_update_owned.profile_id = profile_id_param.to_string();

        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<Goal> {
                diesel::update(goals_dsl.find(goal_id_owned.clone()).filter(goals::profile_id.eq(goal_update_owned.profile_id.clone())))
                    .set(&goal_update_owned)
                    .execute(conn)?;
                Ok(goals_dsl.filter(id.eq(goal_id_owned)).first(conn)?)
            })
            .await
    }

    async fn delete_goal(&self, goal_id_to_delete: String, profile_id_param: &str) -> Result<usize> {
        let profile_id_owned = profile_id_param.to_string();
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<usize> {
                Ok(diesel::delete(goals_dsl.find(goal_id_to_delete).filter(goals::profile_id.eq(profile_id_owned))).execute(conn)?)
            })
            .await
    }

    fn load_allocations_for_non_achieved_goals(&self, profile_id: &str) -> Result<Vec<GoalsAllocation>> {
        self.load_allocations_for_non_achieved_goals_impl(profile_id)
    }

    async fn upsert_goal_allocations(&self, allocations: Vec<GoalsAllocation>, _profile_id: &str) -> Result<usize> {
        let allocations_owned = allocations.clone();

        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<usize> {
                let mut affected_rows = 0;
                for allocation in allocations_owned {
                    affected_rows += diesel::insert_into(goals_allocation::table)
                        .values(&allocation)
                        .on_conflict(goals_allocation::id)
                        .do_update()
                        .set(&allocation)
                        .execute(conn)?;
                }
                Ok(affected_rows)
            })
            .await
    }
}
