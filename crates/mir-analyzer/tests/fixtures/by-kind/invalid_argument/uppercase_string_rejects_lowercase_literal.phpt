===description===
A literal with a lowercase letter is not an `uppercase-string`, also inside a list.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param uppercase-string $code */
function country(string $code): void {}

/** @param list<uppercase-string> $codes */
function countries(array $codes): void {}

country('PL');
country('ISO-3166');
country('pl');
//      ^^^^ InvalidArgument: Argument $code of country() expects 'uppercase-string', got '"pl"'
country('Pl');
//      ^^^^ InvalidArgument: Argument $code of country() expects 'uppercase-string', got '"Pl"'
countries(['PL', 'DE']);
countries(['PL', 'de']);
//        ^^^^^^^^^^^^ InvalidArgument: Argument $codes of countries() expects 'list<uppercase-string>', got 'array{0: "PL", 1: "de"}'
