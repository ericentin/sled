defmodule SledTest do
  use ExUnit.Case
  doctest Sled

  test "db_path open" do
    path = Path.join(System.tmp_dir!(), "SledTestDBDoNotUse")
    File.rm_rf(path)
    assert {:ok, db} = Sled.open(path)
    assert :ok = Sled.insert(db, "hello", "world")
    assert {:ok, "world"} = Sled.get(db, "hello")
  end

  test "config open" do
    path = Path.join(System.tmp_dir!(), "SledTestDBDoNotUse")
    File.rm_rf(path)
    config = Sled.Config.new(path: path)
    assert {:ok, db} = Sled.open(config)

    assert File.exists?(path)
  end
end
