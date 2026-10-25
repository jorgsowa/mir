===description===
Before PHP 8.1 a class may not redeclare an interface constant either, and the message names the interface.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
interface First { const LIMIT = 1; }
interface Second { const LIMIT = 1; }

class BothInterfaces implements First, Second {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot inherit previously-inherited or override constant LIMIT from interface Second [ambiguous-constant]

class Redeclares implements First { const LIMIT = 3; }
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot inherit previously-inherited or override constant LIMIT from interface First [ambiguous-constant]
