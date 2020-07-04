defmodule Sled.ConfigTest do
  use ExUnit.Case
  doctest Sled.Config

  setup_all do
    on_exit(fn ->
      File.rm_rf!("test_config_db")
    end)
  end

  test "config inspect" do
    assert inspect(Sled.Config.new()) =~ ~r/#Sled\.Config<sled::Config\(.*\)>/
  end

  test "config segment_mode" do
    assert_configured(:mode, :low_space, "LowSpace")
    assert_configured(:mode, :high_throughput, "HighThroughput")

    assert_configure_raises(
      :mode,
      :not_a_mode,
      "Erlang error: \"Could not decode field :mode on %SledConfigOptions{}\""
    )
  end

  defp assert_configured(key, value, expected) do
    assert inspect(Sled.Config.new([{key, value}])) =~ "#{key}: #{expected},"
  end

  defp assert_configure_raises(key, value, expected) do
    assert_raise ErlangError, expected, fn -> Sled.Config.new([{key, value}]) end
  end
end
