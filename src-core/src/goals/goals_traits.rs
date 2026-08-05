use crate::errors::Result;
use crate::goals::goals_model::{Goal, GoalsAllocation, NewGoal};
use async_trait::async_trait;

/// Trait for goal repository operations
#[async_trait]
pub trait GoalRepositoryTrait: Send + Sync {
    fn load_goals(&self, profile_id: &str) -> Result<Vec<Goal>>;
    async fn insert_new_goal(&self, new_goal: NewGoal, profile_id: &str) -> Result<Goal>;
    async fn update_goal(&self, goal_update: Goal, profile_id: &str) -> Result<Goal>;
    async fn delete_goal(&self, goal_id_to_delete: String, profile_id: &str) -> Result<usize>;
    fn load_allocations_for_non_achieved_goals(&self, profile_id: &str) -> Result<Vec<GoalsAllocation>>;
    async fn upsert_goal_allocations(&self, allocations: Vec<GoalsAllocation>, profile_id: &str) -> Result<usize>;
}

/// Trait for goal service operations
#[async_trait]
pub trait GoalServiceTrait: Send + Sync {
    fn get_goals(&self, profile_id: &str) -> Result<Vec<Goal>>;
    async fn create_goal(&self, new_goal: NewGoal, profile_id: &str) -> Result<Goal>;
    async fn update_goal(&self, updated_goal_data: Goal, profile_id: &str) -> Result<Goal>;
    async fn delete_goal(&self, goal_id_to_delete: String, profile_id: &str) -> Result<usize>;
    async fn upsert_goal_allocations(&self, allocations: Vec<GoalsAllocation>, profile_id: &str) -> Result<usize>;
    fn load_goals_allocations(&self, profile_id: &str) -> Result<Vec<GoalsAllocation>>;
}
