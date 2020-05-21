defmodule Sled.Native do
  @moduledoc false

  use Rustler, otp_app: :sled, crate: :sled_nif

  def sled_config_new(_options), do: error()
  def sled_config_open(_config), do: error()
  def sled_config_inspect(_config), do: error()

  def sled_open(_db_path), do: error()
  def sled_insert(_db, _k, _v), do: error()
  def sled_get(_db, _k), do: error()

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
