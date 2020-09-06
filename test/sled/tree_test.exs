defmodule Sled.TreeTest do
  use ExUnit.Case
  doctest Sled.Tree

  setup do
    path = Sled.TestHelpers.test_db_name()
    File.rm_rf!(path)

    on_exit(fn ->
      File.rm_rf!(path)
    end)

    {:ok, db: Sled.open(path)}
  end

  test "open tree", %{db: db} do
    assert %Sled.Tree{} = Sled.open_tree(db, "test_tree")
  end

  test "tree inspect", %{db: db} do
    assert inspect(Sled.open_tree(db, "test_tree")) ==
             "#Sled.Tree<db: #Sled<path: #{inspect(db.path)}, ...>, name: \"test_tree\", ...>"
  end

  test "tree insert/remove", %{db: db} do
    tree = Sled.open_tree(db, "test_tree")
    assert nil == Sled.insert(tree, "hello", "world")
    assert nil == Sled.get(db, "hello")
    assert "world" == Sled.remove(tree, "hello")
    assert nil == Sled.get(tree, "hello")
  end

  test "tree checksum", %{db: db} do
    tree = Sled.open_tree(db, "test_tree")
    assert 0 == Sled.checksum(db)
    assert 0 == Sled.checksum(tree)
    assert nil == Sled.insert(tree, "hello", "world")
    assert 0 == Sled.checksum(db)
    assert 4_192_936_109 == Sled.checksum(tree)
  end
end
