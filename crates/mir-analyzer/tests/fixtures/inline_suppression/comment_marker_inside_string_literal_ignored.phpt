===description===
`//`/`#` inside a same-line string literal was mistaken for a real trailing
comment — a plain string statement containing `// @psalm-suppress …` text
wrongly registered a same-line suppression, silencing a genuinely
unrelated issue on the very same physical line.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = "// @psalm-suppress UndefinedClass"; new NoSuchClass();
//                                            ^^^^^^^^^^^ UndefinedClass: Class NoSuchClass does not exist
