# Superposition
Superposition is a cloud configuration and experimentation management platform that allows software teams to manage their configuration via a central location, run multi-variate experiments for different configuration values and evaluate results of these experiments and conclude them accordingly.

The Superposition platform comes with three components:

* **Context-Aware-Configuration** - a flexible configuration management system that supports contextual overrides for configuration keys
* **Experimentation** - a experimentation management system that allows supplying different configuration values to equal-sized cohorts (facilitating A/B testing)
* **Metrics** - a metrics sub-system that interacts with analytics backends to provide supporting metrics that enable conclusions to be drawn from experiments (TBD)

## Detailed documentation
1. [Context-Aware-Configuration](docs/context-aware-configuration.md)
2. [Experimentation](docs/context-aware-configuration.md)
3. [Metrics](docs/metrics.md)
4. [Clients](docs/clients.md)
5. [Local setup](docs/setup.md)

## Key features
* **Admin UI** - Out of the box administration (and tools) UI for configurations and experiments
* **Rich API support** - every action on the platform to manage configurations / experiments is supported with an accompanying API
* **Type/Validation support** - Comprehensive type support using json-schema and custom validator function support for configuration values
* **Multi-tenant support** - a single deployment allows multiple tenants to manage their configurations/experiments in a completely isolated manner
* **Authn/Authz support** - control who can make configuration/experimentation changes