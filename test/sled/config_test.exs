defmodule Sled.ConfigTest do
  use ExUnit.Case
  doctest Sled.Config

  test "config inspect" do
    assert inspect(Sled.Config.new()) =~ ~r/#Sled\.Config<sled::Config\(.*\)>/
  end

  test "config segment_mode" do
    assert_configured(:segment_mode, :linear, "Linear")
    assert_configured(:segment_mode, :gc, "Gc")

    assert_configure_raises(
      :segment_mode,
      :not_a_mode,
      "Erlang error: \"Could not decode field :segment_mode on %SledConfigOptions{}\""
    )
  end

  test "config flush_every_ms" do
    assert_configured(:flush_every_ms, 1234, "Some(1234)")
    assert_configured(:flush_every_ms, false, "None")

    assert_configure_raises(
      :flush_every_ms,
      :not_a_time,
      "Erlang error: \"Could not decode field :flush_every_ms on %SledConfigOptions{}\""
    )
  end

  test "config snapshot_path" do
    assert_configured(:snapshot_path, "my_snapshot_path", "Some(\"my_snapshot_path\")")
    assert_configured(:snapshot_path, false, "None")

    assert_configure_raises(
      :snapshot_path,
      :not_a_path,
      "Erlang error: \"Could not decode field :snapshot_path on %SledConfigOptions{}\""
    )
  end

  defp assert_configured(key, value, expected) do
    assert inspect(Sled.Config.new([{key, value}])) =~ "#{key}: #{expected},"
  end

  defp assert_configure_raises(key, value, expected) do
    assert_raise ErlangError, expected, fn -> Sled.Config.new([{key, value}]) end
  end
end
