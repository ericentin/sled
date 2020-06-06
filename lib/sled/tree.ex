defmodule Sled.Tree do
  @moduledoc "Defines a struct which references a sled Tree."
  @enforce_keys [:ref, :name]
  defstruct ref: nil, name: nil

  @typedoc """
  A handle to a sled Tree.
  """
  @opaque t :: %__MODULE__{ref: reference(), name: binary()}

  @doc """
  Open the sled [Tree](https://docs.rs/sled/0.31.0/sled/struct.Db.html#method.open_tree) accessed via `db`
  with `name`, creating it if it doesn't exist.
  """
  @spec open(Sled.Tree.t(), binary()) :: Sled.Tree.t() | no_return
  def open(db, name) do
    Sled.Native.sled_tree_open(db, name)
  end

  parent = __MODULE__

  defimpl Inspect do
    @impl true
    def inspect(%unquote(parent){} = tree, _opts) do
      "##{unquote(inspect(parent))}<name: \"#{tree.name}\">"
    end
  end
end
