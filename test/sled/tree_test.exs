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

    assert inspect(Sled.Tree.open(db, "test_tree")) ==
             "#Sled.Tree<db: #Sled<path: \"#{db.path}\", ...>, name: \"test_tree\", ...>"
  end
end
