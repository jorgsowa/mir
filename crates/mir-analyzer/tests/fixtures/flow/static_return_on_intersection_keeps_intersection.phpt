===description===
A `static` return called on an `A&B` receiver is the whole intersection, not just the declaring part
===file===
<?php
interface Pool {}
interface Namespaced { public function withSub(string $n): static; }

final class Holder {
    public function __construct(private Pool $pool) {}

    public function f(string $n): void {
        if ($this->pool instanceof Namespaced) {
            $this->pool = $this->pool->withSub($n);
        }
    }
}
