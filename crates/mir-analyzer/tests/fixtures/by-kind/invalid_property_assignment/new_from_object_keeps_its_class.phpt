===description===
`new $obj()` instantiates `$obj`'s class
===file===
<?php
interface Store {}

final class Holder {
    private ?Store $store = null;

    public function reset(Store $s): void {
        $this->store = new $s();
    }
}
