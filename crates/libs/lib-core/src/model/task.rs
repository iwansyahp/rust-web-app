use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use modql::filter::{
	FilterNodes, ListOptions, OpValsBool, OpValsInt64, OpValsString,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// region:    --- Task Types
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Task {
	pub id: i64,
	pub project_id: i64,

	pub title: String,
	pub done: bool,
}

#[derive(Fields, Deserialize)]
pub struct TaskForCreate {
	pub title: String,
	pub project_id: i64,
}

#[derive(Fields, Deserialize, Default)]
pub struct TaskForUpdate {
	pub title: Option<String>,
	pub done: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct TaskFilter {
	project_id: Option<OpValsInt64>,
	title: Option<OpValsString>,
	done: Option<OpValsBool>,
}
// endregion: --- Task Types

// region:    --- TaskBmc
pub struct TaskBmc;

impl DbBmc for TaskBmc {
	const TABLE: &'static str = "task";
}

impl TaskBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		task_c: TaskForCreate,
	) -> Result<i64> {
		base::create::<Self, _>(ctx, mm, task_c).await
	}

	pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
		base::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filter: Option<TaskFilter>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<Task>> {
		base::list::<Self, _, _>(ctx, mm, filter, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: i64,
		task_u: TaskForUpdate,
	) -> Result<()> {
		base::update::<Self, _>(ctx, mm, id, task_u).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
		base::delete::<Self>(ctx, mm, id).await
	}
}
// endregion: --- TaskBmc

// region:    --- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::_dev_utils;
	use crate::model::project::ProjectBmc;
	use crate::model::Error;
	use anyhow::Result;
	use modql::filter::OpValString;
	use serde_json::json;
	use serial_test::serial;

	#[serial]
	#[tokio::test]
	async fn test_create_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title = "test_create_ok title";
		let fx_project_id =
			_dev_utils::seed_project(&ctx, &mm, "test_create_ok project for task ")
				.await?;

		// -- Exec
		let task_c = TaskForCreate {
			project_id: fx_project_id,
			title: fx_title.to_string(),
		};
		let id = TaskBmc::create(&ctx, &mm, task_c).await?;

		// -- Check
		let task = TaskBmc::get(&ctx, &mm, id).await?;
		assert_eq!(task.title, fx_title);

		// -- Clean
		TaskBmc::delete(&ctx, &mm, id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_get_err_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_id = 100;

		// -- Exec
		let res = TaskBmc::get(&ctx, &mm, fx_id).await;

		// -- Check
		assert!(
			matches!(
				res,
				Err(Error::EntityNotFound {
					entity: "task",
					id: 100
				})
			),
			"EntityNotFound not matching"
		);

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_list_all_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_titles = &["test_list_all_ok-task 01", "test_list_all_ok-task 02"];
		let fx_project_id =
			_dev_utils::seed_project(&ctx, &mm, "test_list_all_ok project for task")
				.await?;
		_dev_utils::seed_tasks(&ctx, &mm, fx_project_id, fx_titles).await?;

		// -- Exec
		let filter: TaskFilter = TaskFilter {
			project_id: Some(fx_project_id.into()),
			..Default::default()
		};
		let tasks = TaskBmc::list(&ctx, &mm, Some(filter), None).await?;

		// -- Check
		assert_eq!(tasks.len(), 2, "number of seeded tasks.");

		// -- Clean
		ProjectBmc::delete(&ctx, &mm, fx_project_id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_list_by_title_contains_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_titles = &[
			"test_list_by_title_contains_ok 01",
			"test_list_by_title_contains_ok 02.1",
			"test_list_by_title_contains_ok 02.2",
		];
		let fx_project_id = _dev_utils::seed_project(
			&ctx,
			&mm,
			"test_list_by_title_contains_ok project for task ",
		)
		.await?;
		_dev_utils::seed_tasks(&ctx, &mm, fx_project_id, fx_titles).await?;

		// -- Exec
		let filter = TaskFilter {
			project_id: Some(fx_project_id.into()),
			title: Some(
				OpValString::Contains("by_title_contains_ok 02".to_string()).into(),
			),
			..Default::default()
		};
		let tasks = TaskBmc::list(&ctx, &mm, Some(filter), None).await?;

		// -- Check
		assert_eq!(tasks.len(), 2);

		// -- Cleanup
		ProjectBmc::delete(&ctx, &mm, fx_project_id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_list_with_list_options_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_titles = &[
			"test_list_with_list_options_ok 01",
			"test_list_with_list_options_ok 02.1",
			"test_list_with_list_options_ok 02.2",
		];
		let fx_project_id = _dev_utils::seed_project(
			&ctx,
			&mm,
			"test_list_with_list_options_ok project for task ",
		)
		.await?;
		_dev_utils::seed_tasks(&ctx, &mm, fx_project_id, fx_titles).await?;

		// -- Exec
		let filter: TaskFilter = TaskFilter {
			project_id: Some(fx_project_id.into()),
			..Default::default()
		};
		let list_options: ListOptions = serde_json::from_value(json! ({
			"offset": 0,
			"limit": 2,
			"order_bys": "!title"
		}))?;
		let tasks =
			TaskBmc::list(&ctx, &mm, Some(filter), Some(list_options)).await?;

		// -- Check
		let titles: Vec<String> =
			tasks.iter().map(|t| t.title.to_string()).collect();
		assert_eq!(titles.len(), 2);
		assert_eq!(
			&titles,
			&[
				"test_list_with_list_options_ok 02.2",
				"test_list_with_list_options_ok 02.1"
			]
		);

		// -- Cleanup
		// Will delete associated tasks
		ProjectBmc::delete(&ctx, &mm, fx_project_id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_update_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title = "test_update_ok - task 01";
		let fx_title_new = "test_update_ok - task 01 - new";
		let fx_project_id =
			_dev_utils::seed_project(&ctx, &mm, "test_update_ok project for task")
				.await?;
		let fx_task = _dev_utils::seed_tasks(&ctx, &mm, fx_project_id, &[fx_title])
			.await?
			.remove(0);

		// -- Exec
		TaskBmc::update(
			&ctx,
			&mm,
			fx_task.id,
			TaskForUpdate {
				title: Some(fx_title_new.to_string()),
				..Default::default()
			},
		)
		.await?;

		// -- Check
		let task = TaskBmc::get(&ctx, &mm, fx_task.id).await?;
		assert_eq!(task.title, fx_title_new);

		// -- Clean
		ProjectBmc::delete(&ctx, &mm, fx_project_id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_delete_err_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_id = 100;

		// -- Exec
		let res = TaskBmc::delete(&ctx, &mm, fx_id).await;

		// -- Check
		assert!(
			matches!(
				res,
				Err(Error::EntityNotFound {
					entity: "task",
					id: 100
				})
			),
			"EntityNotFound not matching"
		);

		Ok(())
	}
}
// endregion: --- Tests
