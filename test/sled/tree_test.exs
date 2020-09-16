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
    assert nil == Sled.Tree.insert(db, "hello", "world")
    assert nil == Sled.Tree.insert(tree, "hello", "world2")
    assert "world" == Sled.Tree.get(db, "hello")
    assert "world2" == Sled.Tree.get(tree, "hello")
  end

  test "insert/remove", %{db: db, tree: tree} do
    assert nil == Sled.Tree.insert(db, "hello", "world")
    assert nil == Sled.Tree.insert(tree, "hello", "world2")
    assert "world" == Sled.Tree.remove(db, "hello")
    assert "world2" == Sled.Tree.remove(tree, "hello")
  end

  test "compare_and_swap", %{db: db, tree: tree} do
    assert {:ok, {}} == Sled.Tree.compare_and_swap(db, "hello", nil, "world")
    assert {:ok, {}} == Sled.Tree.compare_and_swap(tree, "hello", nil, "world")
    assert "world" == Sled.Tree.get(db, "hello")
    assert "world" == Sled.Tree.get(tree, "hello")

    assert {:error, {"world", "world2"}} ==
             Sled.Tree.compare_and_swap(tree, "hello", nil, "world2")

    assert {:ok, {}} == Sled.Tree.compare_and_swap(tree, "hello", "world", "world3")
    assert "world3" == Sled.Tree.get(tree, "hello")
    assert {:ok, {}} == Sled.Tree.compare_and_swap(tree, "hello", "world3", nil)
    assert nil == Sled.Tree.get(tree, "hello")
  end

  test "checksum", %{db: db, tree: tree} do
    a = Sled.Tree.checksum(db)
    b = Sled.Tree.checksum(tree)
    assert nil == Sled.Tree.insert(tree, "hello", "world")
    assert a == Sled.Tree.checksum(db)
    assert b != Sled.Tree.checksum(tree)
  end

  test "flush", %{db: db, tree: tree} do
    Sled.Tree.insert(db, "hello", "world")
    assert 0 != Sled.Tree.flush(db)
    assert 0 == Sled.Tree.flush(db)
    Sled.Tree.insert(tree, "hello", "world")
    assert 0 != Sled.Tree.flush(tree)
    assert 0 == Sled.Tree.flush(tree)
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

  test "transaction", %{db: _db, tree: tree} do
    assert {:ok, :result} =
             Sled.Tree.transaction(tree, fn tx_tree ->
               assert nil ==
                        Sled.Tree.Transactional.insert(tx_tree, "hello", "world")

               assert "world" ==
                        Sled.Tree.Transactional.insert(tx_tree, "hello", "world2")

               :result
             end)
  end
end
