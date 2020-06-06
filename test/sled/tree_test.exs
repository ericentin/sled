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

  test "open tree", context do
    assert %Sled.Tree{} = Sled.Tree.open(context.db, "test_tree")
  end

  test "tree inspect" do
    assert db = %Sled{} = Sled.open(Sled.TestHelpers.test_db_name())
    assert "#Sled.Tree<name: \"test_tree\">" = inspect(Sled.Tree.open(db, "test_tree"))
  end
end
