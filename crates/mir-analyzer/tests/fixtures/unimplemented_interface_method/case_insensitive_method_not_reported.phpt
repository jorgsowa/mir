===file===
<?php
interface Serializable {
    public function fooBar(): void;
}
class Impl implements Serializable {
    public function fooBar(): void {}
}
===expect===
