defmodule Sled.Config do
  defmodule Options do
    defstruct path: nil,
              flush_every_ms: nil,
              temporary: nil,
              create_new: nil,
              cache_capacity: nil,
              print_profile_on_drop: nil,
              use_compression: nil,
              compression_factor: nil,
              snapshot_after_ops: nil,
              segment_cleanup_threshold: nil,
              segment_cleanup_skew: nil,
              segment_mode: nil,
              snapshot_path: nil,
              idgen_persist_interval: nil,
              read_only: nil

    @type t :: %__MODULE__{
            path: Path.t(),
            flush_every_ms: integer(),
            temporary: boolean(),
            create_new: boolean(),
            cache_capacity: integer(),
            print_profile_on_drop: boolean(),
            use_compression: boolean(),
            compression_factor: integer(),
            snapshot_after_ops: integer(),
            segment_cleanup_threshold: integer(),
            segment_cleanup_skew: integer(),
            segment_mode: atom(),
            snapshot_path: Path.t(),
            idgen_persist_interval: integer(),
            read_only: boolean()
          }
  end

  @opaque t :: %__MODULE__{ref: reference()}

  @doc false
  defstruct ref: nil

  def new(options \\ %Options{})

  def new(%Options{} = options) do
    %__MODULE__{ref: Sled.Native.sled_config_new(options)}
  end

  def new(options) when is_list(options) do
    new(struct(Sled.Config.Options, options))
  end
end
