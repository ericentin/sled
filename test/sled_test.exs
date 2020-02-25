defmodule SledTest do
  use ExUnit.Case
  doctest Sled

  test "basically works" do
    path = Path.join(System.tmp_dir!(), "SledTestDBDoNotUse")
    File.rm_rf(path)
    assert {:ok, db} = Sled.open(path)
    assert :ok = Sled.insert(db, "hello", "world")
    assert {:ok, "world"} = Sled.get(db, "hello")
  end
end
