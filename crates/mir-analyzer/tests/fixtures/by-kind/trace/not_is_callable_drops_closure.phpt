===description===
A `Closure` is always callable, so `!is_callable()` rules it out; the other
members stay.
===file===
<?php
function iterable_arm(\Closure|iterable $f): void {
    if (!is_callable($f)) {
        /** @trace $f */
        echo 1;
//      ^^^^^^^ Trace: Type of $f is iterable
    } else {
        /** @trace $f */
        echo 1;
//      ^^^^^^^ Trace: Type of $f is Closure|iterable
    }
}

function early_return(\Closure|\Iterator $f): mixed {
    if (is_callable($f)) {
        return $f();
    }
    return $f->valid();
}

final class Holder {
    private \Closure|iterable|null $f = null;

    public function get(): mixed {
        if (null === $this->f) {
            return null;
        }
        if (is_callable($this->f)) {
            return ($this->f)();
        }
        $g = $this->f;
        /** @trace $g */
        return $g;
//      ^^^^^^^^^^ Trace: Type of $g is iterable
    }
}
