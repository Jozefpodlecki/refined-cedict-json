#[cfg(test)]
mod test {

    mod test_is_cjk {
        use crate::utils::is_cjk;

        #[test]
        fn should_return_true() {
            let char = "半".chars().next().unwrap();
            let result = is_cjk(&char);
            assert!(result == true)
        }

        #[test]
        fn should_return_false() {
            let char = "a".chars().next().unwrap();
            let result = is_cjk(&char);
            assert!(result == false)
        }
    }

    mod test_parse_ce_record {
        use crate::utils::parse_ce_record;

        #[test]
        fn should_return_struct() {
            let line = "如泣如訴 如泣如诉 [ru2 qi4 ru2 su4] /lit. as if weeping and complaining (idiom)/fig. mournful (music or singing)/";
            let result = parse_ce_record(&line, 1);
            assert_eq!(result.simplified, "如泣如诉");
            assert_eq!(result.traditional, "如泣如訴");
            assert_eq!(result.wade_giles_pinyin, "ru2 qi4 ru2 su4");
            assert_eq!(
                result.meanings[0],
                "lit. as if weeping and complaining (idiom)"
            );
            assert_eq!(result.meanings[1], "fig. mournful (music or singing)");
            assert_eq!(result.line_number, 1);
        }
    }
}
