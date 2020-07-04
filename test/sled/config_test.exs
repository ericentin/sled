defmodule Sled.ConfigTest do
  use ExUnit.Case
  doctest Sled.Config

  setup_all do
    on_exit(fn ->
      File.rm_rf!("test_config_db")
    end)
  end

  test "config inspect" do
    assert inspect(Sled.Config.new(), limit: :infinity) == "#Sled.Config<...>"
  end

  test "config mode" do
    assert Sled.Config.new(mode: :low_space)
    assert Sled.Config.new(mode: :high_throughput)

    assert_raise ErlangError,
                 "Erlang error: \"Could not decode field :mode on %SledConfigOptions{}\"",
                 fn ->
                   Sled.Config.new(mode: :not_a_mode)
                 end
  end
end
