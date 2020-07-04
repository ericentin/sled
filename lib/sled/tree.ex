defmodule Sled.Tree do
  @moduledoc "Defines a struct which references a sled Tree."

  @derive {Inspect, except: [:ref]}
  @enforce_keys [:ref, :db, :name]
  defstruct ref: nil, db: nil, name: nil

  @typedoc """
  A handle to a sled Tree.
  """
  @opaque t :: %__MODULE__{ref: reference(), db: Sled.t(), name: String.t()}

  @doc """
  Open the sled [Tree](https://docs.rs/sled/0.31.0/sled/struct.Db.html#method.open_tree) accessed
  via `db` with `name`, creating it if it doesn't exist.
  """
  @spec open(Sled.t(), String.t()) :: Sled.Tree.t() | no_return
  def open(db, name) do
    Sled.Native.sled_tree_open(db, name)
  end
end
