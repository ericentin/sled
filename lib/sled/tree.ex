defmodule Sled.Tree do
  @moduledoc "Perform operations on sled trees."

  @derive {Inspect, except: [:ref]}
  @enforce_keys [:ref, :db, :name]
  defstruct ref: nil, db: nil, name: nil

  @typedoc """
  A reference to a sled tenant tree.
  """
  @opaque t :: %__MODULE__{ref: reference(), db: Sled.t(), name: String.t()}

  @typedoc """
  A reference to a sled tree. Passing a `t:Sled.t/0` refers to the default tree for the db, while a
  `t:t/0` references a "tenant" tree.
  """
  @type tree_ref :: t | Sled.t()

  @doc """
  Retrieve the CRC32 of all keys and values in `tree`.

  This is O(N) and locks the underlying tree for the duration of the entire scan.
  """
  @spec checksum(tree_ref) :: integer | no_return
  def checksum(tree) do
    Sled.Native.sled_checksum(tree)
  end

  @doc """
  Synchronously flushes all dirty IO buffers for `tree` and calls fsync.

  If this succeeds, it is guaranteed that all previous writes will be recovered if the system
  crashes. Returns the number of bytes flushed during this call.

  Flushing can take quite a lot of time, and you should measure the performance impact of using it
  on realistic sustained workloads running on realistic hardware.
  """
  @spec flush(tree_ref) :: integer | no_return
  def flush(tree) do
    Sled.Native.sled_flush(tree)
  end

  @doc """
  Insert `value` into `tree` for `key`.

  Returns `nil` if there was no previous value associated with the key.
  """
  @spec insert(tree_ref, binary, binary) :: binary | nil | no_return
  def insert(tree, key, value) do
    Sled.Native.sled_insert(tree, key, value)
  end

  @doc """
  Retrieve the value for `key` from `tree`.

  Returns `nil` if there is no value associated with the key.
  """
  @spec get(tree_ref, binary) :: binary | nil | no_return
  def get(tree, key) do
    Sled.Native.sled_get(tree, key)
  end

  @doc """
  Delete the value for `key` from `tree`.

  Returns `nil` if there is no value associated with the key.
  """
  @spec remove(tree_ref, binary) :: binary | nil | no_return
  def remove(tree, key) do
    Sled.Native.sled_remove(tree, key)
  end

  @doc """
  Compare and swap `old` and `new` values for `key` in `tree`.

  If `old` is `nil`, the value for `key` will be set if it isn't set yet.

  If `new` is `nil`, the value for `key` will be deleted if `old` matches the current value.

  If both `old` and `new` are not `nil`, the value of `key` will be set to `new` if `old` matches
  the current value.

  Upon success, returns `{:ok, {}}`.

  If the operation fails, `{:error, {current, proposed}}` will be returned instead, where
  `current` is the current value for `key` which caused the CAS to fail, and `proposed` is the
  value that was proposed unsuccessfully.
  """
  @spec compare_and_swap(tree_ref, binary, binary | nil, binary | nil) ::
          {:ok, {}}
          | {:error, {current :: binary, proposed :: binary}}
          | no_return
  def compare_and_swap(tree, key, old, new) do
    Sled.Native.sled_compare_and_swap(tree, key, old, new)
  end

  def transaction(tree, fun) do
    tx_tree = Sled.Native.sled_transaction(tree)

    receive do
      {:sled_transaction_status, :start} -> :ok
    end

    do_transaction(tree, fun, tx_tree)
  end

  def do_transaction(tree, fun, tx_tree) do
    try do
      fun.(tx_tree)
    else
      result ->
        Sled.Native.sled_transaction_close(tx_tree)

        receive do
          {:sled_transaction_status, :start} -> do_transaction(tree, fun, tx_tree)
          {:sled_transaction_complete, {:ok, {}}} -> {:ok, result}
          {:sled_transaction_complete, error} -> error
        end
    catch
      {:sled_transaction_status, :start} ->
        do_transaction(tree, fun, tx_tree)

      {:sled_transaction_complete, {:error, reason}} ->
        {:error, reason}

      {:sled_transaction_abort, reason} ->
        Sled.Native.sled_transaction_abort(tx_tree)

        receive do
          {:sled_transaction_complete, {:error, {:abort, nil}}} ->
            {:error, reason}
        end

      class, reason ->
        Sled.Native.sled_transaction_abort(tx_tree)

        receive do
          {:sled_transaction_complete, {:error, {:abort, nil}}} -> :ok
        end

        :erlang.raise(class, reason, __STACKTRACE__)
    end
  end
end
