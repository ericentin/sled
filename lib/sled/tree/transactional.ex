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
    do_call(fn req_ref ->
      Sled.Native.sled_transaction_insert(tree, req_ref, key, value)
    end)
  end

  defp do_call(fun) do
    req_ref = make_ref()
    fun.(:erlang.term_to_binary(req_ref))

    receive do
      {:sled_transaction, ^req_ref, result} -> result
    end
  end
end
