defmodule Sled.Native do
  @moduledoc false

  use Rustler, otp_app: :sled, crate: :sled_nif

  def sled_config_new(_options), do: error()
  def sled_config_open(_config), do: error()

  def sled_open(_path), do: error()

  def sled_tree_open(_db, _name), do: error()
  def sled_tree_drop(_db, _name), do: error()
  def sled_tree_names(_db), do: error()
  def sled_db_checksum(_db), do: error()
  def sled_size_on_disk(_db), do: error()
  def sled_was_recovered(_db), do: error()
  def sled_generate_id(_db), do: error()
  def sled_export(_db), do: error()
  def sled_import(_db, _export), do: error()

  def sled_checksum(_tree), do: error()
  def sled_flush(_tree), do: error()
  def sled_insert(_tree, _k, _v), do: error()
  def sled_get(_tree, _k), do: error()
  def sled_remove(_tree, _k), do: error()
  def sled_compare_and_swap(_tree, _k, _old, _new), do: error()

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
