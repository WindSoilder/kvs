# KVS
My simple kvs implementation for the projects in [talent-plan](https://github.com/pingcap/talent-plan/tree/master/rust)

For now it pass through [project4->part5](https://github.com/pingcap/talent-plan/blob/master/rust/projects/project-4/project.md#user-content-part-5-abstracted-thread-pools)

## Note for different branches
- master branch is hanged on [project4-part8](https://github.com/pingcap/talent-plan/blob/master/rust/projects/project-4/project.md#user-content-part-8-lock-free-readers), and for now the `KvStore` is still using Mutex.  It will implement lock-free reader in the future.
- use_rw_lock branch is hanged on `project4-part8` too, compare to master branch, it use `RwLock` rather than `Mutex` to improve read performance.  Yeah, it support multi-read, one-write scenario.
