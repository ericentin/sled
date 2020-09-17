defmodule Sled.Tree.Transactional do
  @moduledoc "Transactional operations on sled trees."

  @derive {Inspect, except: [:ref]}
  @enforce_keys [:ref, :tree]
  defstruct ref: nil, tree: nil

  @typedoc """
  A reference to a sled tenant tree within a transaction.
  """
  @opaque t :: %__MODULE__{ref: reference(), tree: Sled.Tree.tree_ref()}

  def insert(%Sled.Tree.Transactional{} = tree, key, value) do
    Sled.Native.sled_transaction_insert(tree, key, value)

    receive do
      {:sled_transaction_reply, result} ->
        result

      {tag, _} = result when tag in [:sled_transaction_status, :sled_transaction_complete] ->
        throw(result)
    end
  end

  def abort(reason) do
    throw({:sled_transaction_abort, reason})
  end
end
