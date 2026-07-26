===description===
@psalm-self-out on a call through a CHAINED (2-hop) property-access
receiver (`$this->a->b->method()`) previously silently no-oped —
`extract_any_prop_access` only matched a bare-variable object, so a
receiver reached through one more property hop than the already-fixed
single-hop case fell through to nothing. Retypes via a synthetic
"base->mid_prop" key in the same flat prop_refined map.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Factory {}
class ReadyFactory extends Factory {
    public function build(): string {
        return "built";
    }
}

class Container {
    /** @psalm-self-out ReadyFactory */
    public function prepare(): void {}
}

class Holder {
    public Container $factory;
}

class Wrapper {
    public Holder $holder;

    public function test(): void {
        $this->holder->factory->prepare();
        /** @mir-check $this->holder->factory is ReadyFactory */
        $_ = 1;
        $this->holder->factory->build();
    }
}
===expect===
