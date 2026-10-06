===description===
Passing a value that does not satisfy a function-level @psalm-type alias triggers InvalidArgument
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/**
 * @psalm-type Direction = "north"|"south"|"east"|"west"
 * @param Direction $dir
 */
function move(string $dir): void {}

move("up");
//   ^^^^ InvalidArgument: Argument $dir of move() expects '"north"|"south"|"east"|"west"', got '"up"'
