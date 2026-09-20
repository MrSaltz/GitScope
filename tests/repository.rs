mod common;

use common::TestRepo;
use gitscope::GitScopeError;
use gitscope::repository::Repository;

#[test]
fn nonexistent_path_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does-not-exist");

    let err = Repository::open(&missing).err().expect("must fail");
    assert!(matches!(err, GitScopeError::PathNotFound(p) if p == missing));
}

#[test]
fn directory_without_git_is_not_a_repository() {
    let dir = tempfile::tempdir().unwrap();

    let err = Repository::open(dir.path()).err().expect("must fail");
    assert!(matches!(err, GitScopeError::NotARepository(_)));
}

#[test]
fn regular_file_is_not_a_repository() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("file.txt");
    std::fs::write(&file, "hello").unwrap();

    let err = Repository::open(&file).err().expect("must fail");
    assert!(matches!(err, GitScopeError::NotARepository(_)));
}

#[test]
fn subdirectory_of_a_repository_is_not_searched_upwards() {
    let t = TestRepo::new();
    t.by("Alice", "alice@example.com")
        .write("src/a.rs", "x")
        .commit();

    let err = Repository::open(t.path().join("src"))
        .err()
        .expect("must fail");
    assert!(matches!(err, GitScopeError::NotARepository(_)));
}

#[test]
fn empty_repository_opens_with_no_branches_or_tags() {
    let t = TestRepo::named("empty-project");

    let repo = Repository::open(t.path()).unwrap();
    assert_eq!(repo.name(), "empty-project");
    assert_eq!(repo.branch_count().unwrap(), 0);
    assert_eq!(repo.tag_count().unwrap(), 0);
}

#[test]
fn counts_branches_and_tags() {
    let t = TestRepo::named("my-project");
    t.by("Alice", "alice@example.com")
        .write("a.txt", "a")
        .commit();
    t.create_branch("feature-a");
    t.create_branch("feature-b");
    t.tag("v0.1.0");
    t.annotated_tag("v0.2.0");

    let repo = Repository::open(t.path()).unwrap();
    assert_eq!(repo.name(), "my-project");
    assert_eq!(repo.branch_count().unwrap(), 3);
    assert_eq!(repo.tag_count().unwrap(), 2);
}

#[test]
fn opens_the_dot_git_directory_directly() {
    let t = TestRepo::named("dotgit-project");
    t.by("Alice", "alice@example.com")
        .write("a.txt", "a")
        .commit();

    let repo = Repository::open(t.path().join(".git")).unwrap();
    assert_eq!(repo.name(), "dotgit-project");
    assert_eq!(repo.branch_count().unwrap(), 1);
}

#[test]
fn bare_repository_name_drops_the_git_suffix() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bare-project.git");
    git2::Repository::init_bare(&path).unwrap();

    let repo = Repository::open(&path).unwrap();
    assert_eq!(repo.name(), "bare-project");
}
