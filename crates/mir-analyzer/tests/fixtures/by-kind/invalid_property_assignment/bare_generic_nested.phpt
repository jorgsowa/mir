===description===
bare generic property accepts nested parameterized types
===file===
<?php
/** @template T */
class Container {}

class Wrapper {
//<^^^^^^^^^^^^^^^ MissingConstructor: Class Wrapper has uninitialized properties but no constructor
    private Container $data;

    public function store(): void {
        /** @var Container<array<string, int>> $c */
        $c = new Container();
        $this->data = $c;
    }
}
===expect===
