===description===
G11, property-held array: `$this->data['item'] instanceof Foo` — the
`ShapeBase::Prop` receiver variant of the same access-path machinery, not
just plain variables.
===file===
<?php
class Foo { public function fooOnly(): void {} }
class Bar {}

class Holder {
    /** @var array{item: Foo|Bar} */
    private array $data;

    public function __construct(Foo|Bar $item) {
        $this->data = ['item' => $item];
    }

    public function test(): void {
        if ($this->data['item'] instanceof Foo) {
            $this->data['item']->fooOnly();
        }
    }
}
===expect===
