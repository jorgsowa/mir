===description===
A promoted constructor parameter cannot be made nullable by a `null` default; its type must say so.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
final class Point {
    public function __construct(
        public int $x = null,
//                      ^^^^ CompileError: Cannot use null as default value for parameter $x of type int [default-value-type]
        private readonly ?int $y = null,
        protected int|null $z = null,
        int $plain = null,
//      ^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $plain is never used
    ) {}
}
