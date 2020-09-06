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

  test "insert/get", context do
    assert db = Sled.open(context.path)
    assert nil == Sled.insert(db, "hello", "world")
    assert "world" == Sled.get(db, "hello")
  end

  test "insert/remove", context do
    assert db = Sled.open(context.path)
    assert nil == Sled.insert(db, "hello", "world")
    assert "world" == Sled.remove(db, "hello")
    assert nil == Sled.get(db, "hello")
  end

  test "checksum", context do
    assert db = Sled.open(context.path)
    assert 0 == Sled.checksum(db)
    assert 4_033_561_852 == Sled.db_checksum(db)
    assert nil == Sled.insert(db, "hello", "world")
    assert 4_192_936_109 == Sled.checksum(db)
    assert 2_568_657_029 == Sled.db_checksum(db)
  end
end
