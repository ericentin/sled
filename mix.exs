defmodule Sled.MixProject do
  use Mix.Project

  @version "0.1.0-alpha.2"

  def project do
    [
      app: :sled,
      version: @version,
      elixir: "~> 1.10",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
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
      {:rustler, "~> 0.28"},
      {:ex_doc, "~> 0.29", only: :dev, runtime: false}
    ]
  end

  defp description do
    "An Elixir binding for sled, the champagne of beta embedded databases."
  end

  @sled_github_url "https://github.com/ericentin/sled"

  defp package do
    [
      files: [
        "lib",
        "native/sled_nif/.cargo",
        "native/sled_nif/src",
        "native/sled_nif/Cargo.toml",
        "native/io_uring_test/src",
        "native/io_uring_test/Cargo.toml",
        ".formatter.exs",
        "mix.exs",
        "README.md",
        "LICENSE*"
      ],
      maintainers: ["Eric Entin"],
      licenses: ["Apache-2.0", "MIT"],
      links: %{
        "GitHub" => @sled_github_url,
        "sled" => "https://github.com/spacejam/sled"
      }
    ]
  end
end
