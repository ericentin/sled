defmodule SledTest do
  use ExUnit.Case
  doctest Sled

  setup do
    path = Path.join(System.tmp_dir!(), "SledTestDBDoNotUse")
    File.rm_rf(path)
    {:ok, path: path}
  end

  test "open db_path", context do
    assert %Sled{} = Sled.open!(context.path)

    assert File.exists?(context.path)
  end

  test "open options", context do
    assert %Sled{} = Sled.open!(path: context.path)

    assert File.exists?(context.path)
  end

  test "open config", context do
    assert %Sled{} = Sled.Config.open!(Sled.Config.new!(path: context.path))

    assert File.exists?(context.path)
  end

  test "insert/get", context do
    assert db = Sled.open!(context.path)
    assert :ok = Sled.insert!(db, "hello", "world")
    assert "world" == Sled.get!(db, "hello")
  end

  test "config segment_mode" do
    assert_configured(:segment_mode, :linear, "Linear")
    assert_configured(:segment_mode, :gc, "Gc")
  end

  test "config flush_every_ms" do
    assert_configured(:flush_every_ms, 1234, "Some(1234)")
    assert_configured(:flush_every_ms, false, "None")
  end

  test "config snapshot_path" do
    assert_configured(:snapshot_path, "my_snapshot_path", "Some(\"my_snapshot_path\")")
    assert_configured(:snapshot_path, false, "None")
  end

  defp assert_configured(key, value, expected_value) do
    assert inspect(Sled.Config.new!([{key, value}])) =~ "#{key}: #{expected_value},"
  end
end
