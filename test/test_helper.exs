defmodule Sled.TestHelpers do
  def test_db_name do
    suffix = :crypto.strong_rand_bytes(6) |> Base.encode16()
    Path.join(System.tmp_dir!(), "SledTestDbDoNotUse#{suffix}")
  end
end

ExUnit.start()
