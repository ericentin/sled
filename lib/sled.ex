defmodule Sled do
  @moduledoc """
  An Elixir binding for sled, the champagne of beta embedded databases.

  Example

      {:ok, db} = Sled.open("my_db")
      :ok = Sled.insert(db, "hello", "world")
      {:ok, "world"} = Sled.get(db, "hello")
  """

  @doc """
  Open the db at `db_path`.
  """
  def open(db_path) when is_binary(db_path) do
    Sled.Native.sled_open(db_path)
  end

  @doc """
  Open the db as configured in `config`.
  """
  def open(%Sled.Config{ref: ref}) do
    Sled.Native.sled_config_open(ref)
  end

  @doc """
  Insert into db `db` key `k` and value `v`.
  """
  def insert(db, k, v) do
    Sled.Native.sled_insert(db, k, v)
  end

  @doc """
  Get from db `db` key `k` and value `v`.
  """
  def get(db, k) do
    Sled.Native.sled_get(db, k)
  end
end
