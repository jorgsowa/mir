===description===
A child's native `void` return isn't replaced by the parent's docblock `@return` under `{@inheritdoc}`.
===file===
<?php
class P {
    /** @return \stdClass */
    protected function g() { return new \stdClass(); }
}
class C extends P {
    /** {@inheritdoc} */
    protected function g(): void {
        echo 1;
    }
    public function h(): void {
        $this->g();
    }
}
