defmodule Sled.Native do
  @moduledoc false

  use Rustler, otp_app: :sled, crate: :sled_nif

  def sled_config_new(_options), do: error()
  def sled_config_open(_config), do: error()

  def sled_open(_path), do: error()

  def sled_tree_open(_db, _name), do: error()

  def sled_insert(_tree, _k, _v), do: error()
  def sled_get(_tree, _k), do: error()
  def sled_remove(_tree, _k), do: error()

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
