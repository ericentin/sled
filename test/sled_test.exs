defmodule SledTest do
  use ExUnit.Case
  doctest Sled

  setup do
    path = Path.join(System.tmp_dir!(), "SledTestDbDoNotUse")
    File.rm_rf(path)
    {:ok, path: path}
  end

  test "open db_path", context do
    assert "#Sled<>" == inspect(Sled.open(context.path))

    assert File.exists?(context.path)
  end

  test "open invalid db_path" do
    assert_raise ErlangError,
                 "Erlang error: \"sled::Error::Io(Custom { kind: InvalidInput, error: \\\"data provided contains a nul byte\\\" })\"",
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

  test "insert/del", context do
    assert db = Sled.open(context.path)
    assert nil == Sled.insert(db, "hello", "world")
    assert "world" == Sled.remove(db, "hello")
    assert nil == Sled.get(db, "hello")
  end
end
