defmodule Sled.TreeTest do
  use ExUnit.Case
  doctest Sled.Tree

  setup do
    path = Sled.TestHelpers.test_db_name()
    File.rm_rf!(path)

    on_exit(fn ->
      File.rm_rf!(path)
    end)

    db = Sled.open(path)
    tree_name = "test_tree"

    {:ok, db: db, tree: Sled.open_tree(db, tree_name), tree_name: tree_name}
  end

  test "open tree", %{tree: tree} do
    assert %Sled.Tree{} = tree
  end

  test "tree inspect", %{db: db, tree: tree} do
    assert inspect(tree) ==
             "#Sled.Tree<db: #Sled<path: #{inspect(db.path)}, ...>, name: \"test_tree\", ...>"
  end

  test "insert/get", %{db: db, tree: tree} do
    assert nil == Sled.insert(db, "hello", "world")
    assert nil == Sled.insert(tree, "hello", "world2")
    assert "world" == Sled.get(db, "hello")
    assert "world2" == Sled.get(tree, "hello")
  end

  test "insert/remove", %{db: db, tree: tree} do
    assert nil == Sled.insert(db, "hello", "world")
    assert nil == Sled.insert(tree, "hello", "world2")
    assert "world" == Sled.remove(db, "hello")
    assert "world2" == Sled.remove(tree, "hello")
  end

  test "cas", %{db: db, tree: tree} do
    assert {:ok, {}} == Sled.compare_and_swap(db, "hello", nil, "world")
    assert {:ok, {}} == Sled.compare_and_swap(tree, "hello", nil, "world")
    assert "world" == Sled.get(db, "hello")
    assert "world" == Sled.get(tree, "hello")
    assert {:error, {"world", "world2"}} == Sled.compare_and_swap(tree, "hello", nil, "world2")
    assert {:ok, {}} == Sled.compare_and_swap(tree, "hello", "world", "world3")
    assert "world3" == Sled.get(tree, "hello")
    assert {:ok, {}} == Sled.compare_and_swap(tree, "hello", "world3", nil)
    assert nil == Sled.get(tree, "hello")
  end

  test "checksum", %{db: db, tree: tree} do
    a = Sled.checksum(db)
    b = Sled.checksum(tree)
    assert nil == Sled.insert(tree, "hello", "world")
    assert a == Sled.checksum(db)
    assert b != Sled.checksum(tree)
  end

  test "flush", %{db: db, tree: tree} do
    Sled.insert(db, "hello", "world")
    assert 0 != Sled.flush(db)
    assert 0 == Sled.flush(db)
    Sled.insert(tree, "hello", "world")
    assert 0 != Sled.flush(tree)
    assert 0 == Sled.flush(tree)
  end

  test "drop_tree", %{db: db, tree_name: tree_name} do
    assert false == Sled.drop_tree(db, "uncreated_tree")
    assert true == Sled.drop_tree(db, tree_name)
    assert false == Sled.drop_tree(db, tree_name)
  end

  test "tree_names", %{db: db, tree_name: tree_name} do
    tree_names = Sled.tree_names(db)
    assert tree_name in tree_names
    assert length(tree_names) == 2
  end
end
