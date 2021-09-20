### Enhanced CEDict Json

Converts the CC-CEDICT dictionary format file into a JSON file and includes additional information for every entry

## Examples

```
地窖 地窖 [di4 jiao4] /cellar/basement/
```

becomes

```json
{
    "simplified": "地窖",
    "details": [
        {
            "simplified": "地窖",
            "traditional": "地窖",
            "wade_giles_pinyin": "di4 jiao4",
            "pinyin": "di4 jiao4",
            "breakdown": [
                {
                    "simplified": "地",
                    "traditional": "地",
                    "wade_giles_pinyin": "di4",
                    "pinyin": "di4",
                    "stroke_count": 0,
                    "decomposition": {
                        "once": [],
                        "radical": [],
                        "graphical": []
                    }
                },
                {
                    "simplified": "窖",
                    "traditional": "窖",
                    "wade_giles_pinyin": "jiao4",
                    "pinyin": "jiao4",
                    "stroke_count": 0,
                    "decomposition": {
                        "once": [],
                        "radical": [],
                        "graphical": []
                    }
                }
            ],
            "meanings": [
                {
                    "type": "noun",
                    "value": "cellar"
                },
                {
                    "type": "noun",
                    "value":  "basement"   
                }
            ]
        }
    ],
    "tags": ["Parts of buildings: rooms", "Parts of buildings: floors & parts of floors"]
}
```