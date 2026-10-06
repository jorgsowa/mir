===description===
backed enum ::from() and ::tryFrom() accept one argument without TooManyArguments
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Color: string {
    case Red = 'r';
    case Green = 'g';
}

enum Priority: int {
    case Low = 1;
    case High = 2;
}

$c = Color::from('r');
$t = Color::tryFrom('x');
$p = Priority::from(1);
$q = Priority::tryFrom(99);
