===description===
bare generic property accepts nested parameterized types
===file===
<?php
/** @template T */
class Container {}

class Wrapper {
    private Container $data;

    public function store(): void {
        /** @var Container<array<string, int>> $c */
        $c = new Container();
        $this->data = $c;
    }
}
===expect===
