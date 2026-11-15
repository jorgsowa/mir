===description===
`unserialize()` of stored data without `allowed_classes` can instantiate any
class even when the data isn't proven to come from the request.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Cache { public function get(string $key): string; }

function load(Cache $cache, PDO $db): void {
    $cart = unserialize($cache->get('cart'));
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnsafeUnserialize: unserialize() of non-literal data without allowed_classes can instantiate any class
    $row = $db->query('SELECT prefs FROM users')->fetchColumn();
    $prefs = unserialize((string) $row, ['allowed_classes' => true]);
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnsafeUnserialize: unserialize() of non-literal data without allowed_classes can instantiate any class
    var_dump($cart, $prefs);
}
