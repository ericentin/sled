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

  test "was_recovered", context do
    assert db = Sled.open(context.path)
    refute Sled.was_recovered(db)

    # Since there's no way to force a resource to be dropped, and a sled DB can only be open from
    # one process, we create the db from a separate VM in order to open it a second time from our
    # tests.
    try do
      {_stdout, 0} =
        System.cmd(
          "mix",
          [
            "run",
            "--no-compile",
            "--no-deps-check",
            "--no-archives-check",
            "--no-start",
            "--require",
            "test/was_recovered_helper.exs"
          ],
          into: IO.stream(:stdio, :line),
          env: [{"MIX_ENV", "test"}],
          stderr_to_stdout: true
        )

      assert db2 = Sled.open("test_recovered_db")
      assert Sled.was_recovered(db2)
    after
      File.rm_rf!("test_recovered_db")
    end
  end

  test "generate_id", context do
    db = Sled.open(context.path)
    a = Sled.generate_id(db)
    assert is_integer(a)
    b = Sled.generate_id(db)
    assert is_integer(b)
    assert a != b
  end

  test "export", context do
    db = Sled.open(context.path)
    Sled.insert(db, "hello", "world")
    Sled.insert(db, "hello2", "world2")

    assert [{"tree", "__sled__default", [["hello", "world"], ["hello2", "world2"]]}] ==
             Sled.export(db)
  end

  test "import", context do
    db = Sled.open(context.path)
    Sled.insert(db, "hello", "world")
    Sled.insert(db, "hello2", "world2")
    export = Sled.export(db)

    path = Sled.TestHelpers.test_db_name()

    try do
      db2 = Sled.open(path)
      assert :ok == Sled.import(db2, export)
      assert "world" = Sled.get(db2, "hello")
      assert "world2" = Sled.get(db2, "hello2")
    after
      File.rm_rf!(path)
    end
  end
end
