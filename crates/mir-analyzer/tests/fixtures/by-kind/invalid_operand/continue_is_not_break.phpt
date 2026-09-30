===description===
Continue is not break
===file===
<?php
                    switch(2) {
                        case 2:
                            echo "two
";
                            continue 2;
//                          ^^^^^^^^ ParseError: Parse error: Cannot 'continue' 2 levels
                    }
===expect===
