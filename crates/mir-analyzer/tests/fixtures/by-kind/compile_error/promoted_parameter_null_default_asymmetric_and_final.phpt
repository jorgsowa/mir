===description===
A parameter promoted only by `private(set)` or `final` is promoted too, so its `null` default is the compile error, not the deprecation.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Point {
    public function __construct(
        private(set) int $x = null,
//                            ^^^^ CompileError: Cannot use null as default value for parameter $x of type int [default-value-type]
        final int $y = null,
//                     ^^^^ CompileError: Cannot use null as default value for parameter $y of type int [default-value-type]
    ) {}
}
