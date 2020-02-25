defmodule Sled.MixProject do
  use Mix.Project

  @version "0.1.0-alpha"

  def project do
    [
      app: :sled,
      version: @version,
      elixir: "~> 1.10",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      compilers: [:rustler] ++ Mix.compilers(),
      rustler_crates: [sled_nif: [mode: rustc_mode(Mix.env())]],
      description: description(),
      package: package(),
      source_url: "https://github.com/ericentin/sled",
      docs: [
        main: "Sled",
        extras: ["README.md"],
        source_ref: "v#{@version}"
      ]
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.21.0"},
      {:ex_doc, "~> 0.14", only: :dev, runtime: false}
    ]
  end

  defp description do
    "An Elixir binding for Sled, the champagne of beta embedded databases."
  end

  defp package do
    [
      files: [
        "lib",
        "native/sled_nif/.cargo",
        "native/sled_nif/src",
        "native/sled_nif/Cargo.toml",
        ".formatter.exs",
        "mix.exs",
        "README.md"
      ],
      maintainers: ["Eric Entin"],
      licenses: ["Apache-2.0", "MIT"],
      links: %{
        "GitHub" => "https://github.com/ericentin/sled",
        "Sled" => "https://github.com/spacejam/sled"
      }
    ]
  end

  defp rustc_mode(:prod), do: :release
  defp rustc_mode(_), do: :debug
end
