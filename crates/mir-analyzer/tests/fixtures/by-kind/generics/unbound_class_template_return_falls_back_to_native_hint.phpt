===description===
A bare receiver leaves a class `@template T` unbound; a method returning `@return T`
with a native hint yields the native type to the caller. Bound receivers still
substitute, and a template in the caller's own scope stays raw.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Config {}
final class SpanConfig extends Config {}

/** @template T */
final class Configurator {
    /** @return T */
    public function resolve(string $scope): Config { return new SpanConfig(); }

    /** @return T|null */
    public function maybe(string $scope): ?Config { return null; }

    /** @return T */
    public function viaSelf(): Config {
        $r = $this->resolve('x');
        /** @mir-check $r is T */
        return $r;
    }
}

/** @template T of Config */
final class BoundConfigurator {
    /** @return T */
    public function resolve(): Config { return new SpanConfig(); }
}

final class Holder {
    private Config $config;
    private ?Config $maybe = null;

    public function __construct(?Configurator $c = null, ?BoundConfigurator $b = null) {
        $this->config = $c ? $c->resolve('x') : new SpanConfig();
        $this->maybe = $c?->maybe('x');
        if ($b !== null) {
            $r = $b->resolve();
            /** @mir-check $r is Config */
            $this->config = $r;
        }
    }

    /** @param Configurator<SpanConfig> $c */
    public function bound(Configurator $c): SpanConfig {
        $r = $c->resolve('x');
        /** @mir-check $r is SpanConfig */
        return $r;
    }

    public function bare(Configurator $c): Config {
        $r = $c->resolve('x');
        /** @mir-check $r is Config */
        return $r;
    }

    public function bareNullable(Configurator $c): Config {
        $r = $c->maybe('x');
        /** @mir-check $r is Config|null */
        return $r;
//      ^^^^^^^^^^ NullableReturnStatement: Return type 'Config|null' is not compatible with declared 'Config'
    }
}
