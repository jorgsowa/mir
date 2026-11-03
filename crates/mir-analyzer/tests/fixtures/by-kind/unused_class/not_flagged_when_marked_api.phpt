===description===
`@api` / `@psalm-api` class-likes are used from outside the analyzed code
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
/** @api */
class PublicClient {}

/** @psalm-api */
interface PublicContract {}

/** @api */
trait PublicHelpers {}

/** @api */
enum PublicStatus { case On; }

class Internal {}
//<^^^^^^^^^^^^^^^^^ UnusedClass: Class Internal is never referenced
