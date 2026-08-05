use crate::errors::Result;
use crate::goals::goals_model::{Goal, GoalsAllocation, NewGoal};
use crate::goals::goals_traits::{GoalRepositoryTrait, GoalServiceTrait};
use async_trait::async_trait;
use std::sync::Arc;

pub struct GoalService<T: GoalRepositoryTrait> {
    goal_repo: Arc<T>,
}

impl<T: GoalRepositoryTrait> GoalService<T> {
    pub fn new(goal_repo: Arc<T>) -> Self {
        GoalService { goal_repo }
    }
}

#[async_trait]
impl<T: GoalRepositoryTrait + Send + Sync> GoalServiceTrait for GoalService<T> {
    fn get_goals(&self, profile_id: &str) -> Result<Vec<Goal>> {
        self.goal_repo.load_goals(profile_id)
    }

    async fn create_goal(&self, new_goal: NewGoal, profile_id: &str) -> Result<Goal> {
        self.goal_repo.insert_new_goal(new_goal, profile_id).await
    }

    async fn update_goal(&self, updated_goal_data: Goal, profile_id: &str) -> Result<Goal> {
        self.goal_repo.update_goal(updated_goal_data, profile_id).await
    }

    async fn delete_goal(&self, goal_id_to_delete: String, profile_id: &str) -> Result<usize> {
        self.goal_repo.delete_goal(goal_id_to_delete, profile_id).await
    }

    async fn upsert_goal_allocations(&self, allocations: Vec<GoalsAllocation>, profile_id: &str) -> Result<usize> {
        self.goal_repo.upsert_goal_allocations(allocations, profile_id).await
    }

    fn load_goals_allocations(&self, profile_id: &str) -> Result<Vec<GoalsAllocation>> {
        self.goal_repo.load_allocations_for_non_achieved_goals(profile_id)
    }
}
