defmodule Sled.Native do
  @moduledoc false

  defmodule Helpers do
    @moduledoc false

    def rustc_mode(:prod), do: :release
    def rustc_mode(_), do: :debug

    def features do
      if Application.get_env(:sled, :io_uring, false) or io_uring_supported?() do
        ["io_uring"]
      else
        []
      end
    end

    defp io_uring_supported?() do
      is_linux?() and cargo_present?() and io_uring_test_success?()
    end

    defp is_linux? do
      :os.type() == {:unix, :linux}
    end

    defp cargo_present? do
      not is_nil(System.find_executable("cargo"))
    end

    defp io_uring_test_success? do
      case System.cmd("cargo", ["run"],
             stderr_to_stdout: true,
             cd: Path.join([__DIR__, "..", "..", "native", "io_uring_test"])
           ) do
        {_, 0} ->
          true

        {_, 1} ->
          false

        {stdout, exit_status} ->
          io_uring_detection_error(stdout, exit_status)

          false
      end
    end

    defp io_uring_detection_error(stdout, exit_status) do
      Mix.Shell.IO.error([
        """
        Unexpected error determining if io_uring should be enabled.
        Please open an issue at #{Mix.Project.config()[:package][:links]["GitHub"]} and include the following log. Thanks!
        stdout:
        """,
        stdout,
        "exited with status: #{exit_status}"
      ])
    end
  end

  use Rustler,
    otp_app: :sled,
    crate: :sled_nif,
    mode: Helpers.rustc_mode(Mix.env()),
    features: Helpers.features()

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
