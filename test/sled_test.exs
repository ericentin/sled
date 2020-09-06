defmodule SledTest do
  use ExUnit.Case
  doctest Sled

  setup_all do
    on_exit(fn ->
      File.rm_rf!("test_db")
      File.rm_rf!("test_default_db")
    end)
  end

  setup do
    path = Sled.TestHelpers.test_db_name()
    File.rm_rf!(path)

    on_exit(fn ->
      File.rm_rf!(path)
    end)

    {:ok, path: path}
  end

  test "open db_path", context do
    assert %Sled{} = Sled.open(context.path)

    assert File.exists?(context.path)
  end

  test "db inspect", context do
    assert inspect(Sled.open(context.path)) == "#Sled<path: #{inspect(context.path)}, ...>"
  end

  test "open invalid db_path" do
    assert_raise ErlangError,
                 ~r/Erlang error: \"sled::Error::Io\(Custom { kind: InvalidInput, error: .*/,
                 fn -> Sled.open("\0") end
  end

  test "open options", context do
    assert %Sled{} = Sled.open(path: context.path)

    assert File.exists?(context.path)
  end

  test "open config", context do
    assert %Sled{} = Sled.Config.open(Sled.Config.new(path: context.path))

    assert File.exists?(context.path)
  end

  test "db_checksum", context do
    assert db = Sled.open(context.path)
    db_checksum = Sled.db_checksum(db)
    Sled.insert(db, "hello", "world")
    assert db_checksum != Sled.db_checksum(db)
  end

  test "size_on_disk", context do
    assert db = Sled.open(context.path)
    size_on_disk = Sled.size_on_disk(db)
    Sled.insert(db, "hello", :crypto.strong_rand_bytes(1000))
    Sled.flush(db)
    assert size_on_disk != Sled.size_on_disk(db)
  end
end
