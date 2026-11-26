//! Original starter presentations for File › New.
//!
//! Each deck is a few slides of our own text on one of our themes, written twice: in English and
//! in Spanish, so a deck opened from a Spanish interface arrives in Spanish. Nothing here is copied
//! from another product's templates.

use std::sync::Arc;

use lamina_geom::Size;
use lamina_model::text::TextBody;
use lamina_model::{Presentation, defaults};
use serde_json::Value;

use crate::Result;
use crate::cmd::{bad, slide::find_layout};

/// The language a starter's text is written in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    Es,
}

impl Lang {
    /// `"es"`, `"es-ES"` and `"es_MX.UTF-8"` are Spanish; anything else is English.
    pub fn parse(code: &str) -> Lang {
        let code = code.trim().to_ascii_lowercase();
        if code == "es" || code.starts_with("es-") || code.starts_with("es_") { Lang::Es } else { Lang::En }
    }
}

/// One slide of a starter: a layout, a title, text for each non-title placeholder in order, and
/// optional speaker notes.
pub struct SlideSeed {
    pub layout: &'static str,
    pub title: &'static str,
    pub bodies: &'static [&'static str],
    pub notes: &'static str,
}

/// A starter's words in one language.
pub struct Text {
    pub name: &'static str,
    pub summary: &'static str,
    pub slides: &'static [SlideSeed],
}

/// A starter deck shown in the template gallery. `id` is the English name in lower case; it is what
/// scripts pass to `file.new` whatever language the deck is written in.
pub struct Starter {
    pub id: &'static str,
    pub theme: &'static str,
    pub en: Text,
    pub es: Text,
}

impl Starter {
    /// The deck's words in `lang`.
    pub fn text(&self, lang: Lang) -> &Text {
        match lang {
            Lang::En => &self.en,
            Lang::Es => &self.es,
        }
    }
}

const fn seed(layout: &'static str, title: &'static str, bodies: &'static [&'static str], notes: &'static str) -> SlideSeed {
    SlideSeed { layout, title, bodies, notes }
}

const STARTERS: &[Starter] = &[
    Starter {
        id: "briefing",
        theme: "Harbor",
        en: Text {
            name: "Briefing",
            summary: "A weekly check-in: what changed, what's stuck, and who does the next thing",
            slides: &[
                seed("title", "Weekly briefing", &["What changed, what's stuck, and what we do next"], "Open with what changed since last week."),
                seed(
                    "titleAndContent",
                    "This week's agenda",
                    &["Shipped since last time\n\tOne concrete change\nStill open\n\tThe decision we need\nNext\n\tWho does it, and by when"],
                    "",
                ),
                seed("sectionHeader", "Where we are", &["A short picture before the details"], ""),
                seed(
                    "twoContent",
                    "Done and next",
                    &[
                        "Done\n\tThe change is in people's hands\n\tWe wrote down what we learned",
                        "Next\n\tPick the one risk to retire\n\tSay who owns it",
                    ],
                    "",
                ),
                seed("title", "See you next week", &["Bring one number and one question"], ""),
            ],
        },
        es: Text {
            name: "Resumen semanal",
            summary: "Un repaso semanal: qué ha cambiado, qué está atascado y quién hace lo siguiente",
            slides: &[
                seed(
                    "title",
                    "Resumen semanal",
                    &["Qué ha cambiado, qué está atascado y qué hacemos ahora"],
                    "Empieza por lo que ha cambiado desde la semana pasada.",
                ),
                seed(
                    "titleAndContent",
                    "Orden del día",
                    &[
                        "Entregado desde la última vez\n\tUn cambio concreto\nSigue abierto\n\tLa decisión que necesitamos\nLo siguiente\n\tQuién lo hace y para cuándo",
                    ],
                    "",
                ),
                seed("sectionHeader", "Dónde estamos", &["Una imagen breve antes de los detalles"], ""),
                seed(
                    "twoContent",
                    "Hecho y por hacer",
                    &[
                        "Hecho\n\tEl cambio ya está en manos de la gente\n\tDejamos por escrito lo que aprendimos",
                        "Por hacer\n\tElegir el riesgo que queremos quitar de en medio\n\tDecir quién se encarga",
                    ],
                    "",
                ),
                seed("title", "Hasta la semana que viene", &["Trae un número y una pregunta"], ""),
            ],
        },
    },
    Starter {
        id: "lesson",
        theme: "Meadow",
        en: Text {
            name: "Lesson",
            summary: "A short class that ends with the learner explaining the idea",
            slides: &[
                seed("title", "How a lock works", &["A short lesson for people new to the idea"], ""),
                seed(
                    "titleAndContent",
                    "By the end",
                    &["You can say what a lock protects\n\tAnd what it does not\nYou can sketch one in three steps"],
                    "",
                ),
                seed("sectionHeader", "The idea", &["One metaphor, then the real thing"], ""),
                seed(
                    "titleAndContent",
                    "Three steps",
                    &["Name the thing being shared\nDecide who may change it\nLeave it so the next person can tell"],
                    "",
                ),
                seed("title", "Try it", &["Explain a lock to someone who has never seen one"], ""),
            ],
        },
        es: Text {
            name: "Clase",
            summary: "Una clase corta que termina con el alumno explicando la idea",
            slides: &[
                seed("title", "Cómo funciona una cerradura", &["Una clase breve para quien descubre la idea"], ""),
                seed(
                    "titleAndContent",
                    "Al terminar",
                    &["Sabrás decir qué protege una cerradura\n\ty qué no protege\nPodrás dibujar una en tres pasos"],
                    "",
                ),
                seed("sectionHeader", "La idea", &["Primero una metáfora, luego lo real"], ""),
                seed(
                    "titleAndContent",
                    "Tres pasos",
                    &["Nombra lo que se comparte\nDecide quién puede cambiarlo\nDéjalo de modo que el siguiente lo note"],
                    "",
                ),
                seed("title", "Pruébalo", &["Explícale una cerradura a alguien que nunca ha visto una"], ""),
            ],
        },
    },
    Starter {
        id: "pitch",
        theme: "Ember",
        en: Text {
            name: "Pitch",
            summary: "A ten-minute conversation about one promise, not a tour of features",
            slides: &[
                seed("title", "A quieter inbox", &["Ten minutes. One promise."], ""),
                seed(
                    "titleAndContent",
                    "The mess",
                    &[
                        "People miss the one message that mattered\n\tIt arrived between newsletters\nThey spend the morning sorting\n\tAnd still feel behind",
                    ],
                    "",
                ),
                seed("sectionHeader", "A smaller promise", &["Show the one message. Hide the rest until asked."], ""),
                seed(
                    "comparison",
                    "Today and after",
                    &[
                        "Today",
                        "The important note is buried\nFinding it takes a search",
                        "After",
                        "It sits at the top until you deal with it\nEverything else waits",
                    ],
                    "",
                ),
                seed("title", "The ask", &["Twenty people, two weeks, one question: did the right note surface?"], ""),
            ],
        },
        es: Text {
            name: "Propuesta",
            summary: "Una conversación de diez minutos sobre una promesa, no un recorrido por funciones",
            slides: &[
                seed("title", "Un buzón más tranquilo", &["Diez minutos. Una promesa."], ""),
                seed(
                    "titleAndContent",
                    "El problema",
                    &["La gente se pierde el mensaje que importaba\n\tLlegó entre boletines\nPasan la mañana ordenando\n\ty aun así van con retraso"],
                    "",
                ),
                seed("sectionHeader", "Una promesa más pequeña", &["Mostrar el único mensaje. Esconder el resto hasta que se pida."], ""),
                seed(
                    "comparison",
                    "Hoy y después",
                    &[
                        "Hoy",
                        "La nota importante está enterrada\nEncontrarla exige una búsqueda",
                        "Después",
                        "Se queda arriba hasta que la atiendes\nTodo lo demás espera",
                    ],
                    "",
                ),
                seed("title", "La petición", &["Veinte personas, dos semanas, una pregunta: ¿salió a la superficie el mensaje correcto?"], ""),
            ],
        },
    },
    Starter {
        id: "field notes",
        theme: "Coral Reef",
        en: Text {
            name: "Field Notes",
            summary: "What an outing set out to see, and what it actually found",
            slides: &[
                seed("title", "Saturday on the ridge", &["What we went to see, and what we found"], ""),
                seed("titleAndContent", "Plan", &["Leave at dawn\n\tCold, clear, little wind\nWalk the west path\nBe back before dusk"], ""),
                seed("sectionHeader", "What we saw", &["Notes taken on the spot"], ""),
                seed(
                    "twoContent",
                    "Expected and found",
                    &[
                        "Expected\n\tA clear view from the ridge\n\tThe creek on the map",
                        "Found\n\tLow cloud on the ridge\n\tThe creek, and a better pencil",
                    ],
                    "",
                ),
                seed("title", "Pack list for next time", &["Warmer gloves. A second pencil. The map that shows the creek."], ""),
            ],
        },
        es: Text {
            name: "Cuaderno de campo",
            summary: "Lo que una salida quería ver y lo que encontró de verdad",
            slides: &[
                seed("title", "Sábado en la cresta", &["Lo que fuimos a ver y lo que encontramos"], ""),
                seed(
                    "titleAndContent",
                    "Plan",
                    &["Salir al amanecer\n\tFrío, despejado, poco viento\nSubir por el camino del oeste\nVolver antes del anochecer"],
                    "",
                ),
                seed("sectionHeader", "Lo que vimos", &["Notas tomadas sobre la marcha"], ""),
                seed(
                    "twoContent",
                    "Esperado y encontrado",
                    &[
                        "Esperado\n\tUna vista despejada desde la cresta\n\tEl arroyo del mapa",
                        "Encontrado\n\tNubes bajas en la cresta\n\tEl arroyo, y un lápiz mejor",
                    ],
                    "",
                ),
                seed("title", "Qué llevar la próxima vez", &["Guantes más gruesos. Un segundo lápiz. El mapa donde sí sale el arroyo."], ""),
            ],
        },
    },
    Starter {
        id: "quarterly",
        theme: "Paper",
        en: Text {
            name: "Quarterly",
            summary: "A report meant to be read aloud in half an hour",
            slides: &[
                seed("title", "Spring report", &["Read aloud in thirty minutes"], ""),
                seed(
                    "titleAndContent",
                    "Headlines",
                    &["The waiting list got shorter\n\tBy the amount we promised\nTwo things slipped\n\tBoth have a new date"],
                    "",
                ),
                seed("sectionHeader", "The numbers", &["Only the ones that change a decision"], ""),
                seed("titleAndContent", "What we will stop", &["The report nobody reads\nThe meeting that repeats the report"], ""),
                seed("title", "Questions we want", &["Where should the next quarter refuse work?"], ""),
            ],
        },
        es: Text {
            name: "Informe trimestral",
            summary: "Un informe pensado para leerse en voz alta en media hora",
            slides: &[
                seed("title", "Informe de primavera", &["Para leer en voz alta en treinta minutos"], ""),
                seed(
                    "titleAndContent",
                    "Titulares",
                    &["La lista de espera se acortó\n\tLo que habíamos prometido\nDos cosas se retrasaron\n\tLas dos tienen fecha nueva"],
                    "",
                ),
                seed("sectionHeader", "Los números", &["Solo los que cambian una decisión"], ""),
                seed("titleAndContent", "Lo que vamos a dejar", &["El informe que nadie lee\nLa reunión que repite el informe"], ""),
                seed("title", "Preguntas que queremos", &["¿Dónde debería el próximo trimestre rechazar trabajo?"], ""),
            ],
        },
    },
    Starter {
        id: "evening talk",
        theme: "Nocturne",
        en: Text {
            name: "Evening Talk",
            summary: "A public talk the audience can check on the way home",
            slides: &[
                seed("title", "Lights out", &["A talk about how cities sleep"], ""),
                seed("titleAndContent", "Road map", &["Why the sky looks orange\nWhat a dark street is for\nOne change a block can make"], ""),
                seed("sectionHeader", "Look up", &["The part you can check on the way home"], ""),
                seed(
                    "titleAndContent",
                    "One change",
                    &["Shield the lamp so light goes down\n\tNot into bedroom windows\nLeave a path you can still walk"],
                    "",
                ),
                seed("title", "Walk home", &["Count the lamps you can see from your pillow"], ""),
            ],
        },
        es: Text {
            name: "Charla nocturna",
            summary: "Una charla pública que el público puede comprobar de camino a casa",
            slides: &[
                seed("title", "Luces fuera", &["Una charla sobre cómo duermen las ciudades"], ""),
                seed(
                    "titleAndContent",
                    "Guion",
                    &["Por qué el cielo se ve naranja\nPara qué sirve una calle oscura\nUn cambio que puede hacer una manzana"],
                    "",
                ),
                seed("sectionHeader", "Mira arriba", &["La parte que puedes comprobar de camino a casa"], ""),
                seed(
                    "titleAndContent",
                    "Un cambio",
                    &[
                        "Apantalla la farola para que la luz vaya hacia abajo\n\tNo hacia las ventanas de los dormitorios\nDeja un camino por el que aún se pueda andar",
                    ],
                    "",
                ),
                seed("title", "Camino a casa", &["Cuenta las farolas que ves desde la almohada"], ""),
            ],
        },
    },
    Starter {
        id: "kickoff",
        theme: "Lámina",
        en: Text {
            name: "Kickoff",
            summary: "The first meeting of a project: the goal, who does what, and the first week",
            slides: &[
                seed("title", "Project kickoff", &["The goal, the people and the first week"], "Say the goal in one sentence before anything else."),
                seed(
                    "titleAndContent",
                    "The goal",
                    &[
                        "What we want to be true in three months\n\tOne sentence, no adjectives\nWhat we are not doing\n\tSaid out loud so nobody assumes it",
                    ],
                    "",
                ),
                seed("sectionHeader", "Who does what", &["One name for every piece of work"], ""),
                seed(
                    "twoContent",
                    "Roles and rhythm",
                    &[
                        "Roles\n\tWho decides\n\tWho builds\n\tWho tells everyone else",
                        "Rhythm\n\tA short check-in twice a week\n\tOne written update on Friday",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "The first week",
                    &["Monday: agree the goal\nWednesday: first thing in front of someone\nFriday: what we learned, in writing"],
                    "",
                ),
                seed("title", "Questions before we start", &["Anything unclear now costs less than it will in a month"], ""),
            ],
        },
        es: Text {
            name: "Arranque de proyecto",
            summary: "La primera reunión de un proyecto: el objetivo, quién hace qué y la primera semana",
            slides: &[
                seed(
                    "title",
                    "Arranque del proyecto",
                    &["El objetivo, las personas y la primera semana"],
                    "Di el objetivo en una frase antes que nada.",
                ),
                seed(
                    "titleAndContent",
                    "El objetivo",
                    &[
                        "Lo que queremos que sea cierto dentro de tres meses\n\tUna frase, sin adjetivos\nLo que no vamos a hacer\n\tDicho en voz alta para que nadie lo dé por hecho",
                    ],
                    "",
                ),
                seed("sectionHeader", "Quién hace qué", &["Un nombre para cada trabajo"], ""),
                seed(
                    "twoContent",
                    "Papeles y ritmo",
                    &[
                        "Papeles\n\tQuién decide\n\tQuién construye\n\tQuién se lo cuenta a los demás",
                        "Ritmo\n\tUn repaso breve dos veces por semana\n\tUna actualización por escrito el viernes",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "La primera semana",
                    &["Lunes: acordar el objetivo\nMiércoles: la primera cosa delante de alguien\nViernes: lo que aprendimos, por escrito"],
                    "",
                ),
                seed("title", "Preguntas antes de empezar", &["Lo que no está claro hoy cuesta menos que dentro de un mes"], ""),
            ],
        },
    },
    Starter {
        id: "workshop",
        theme: "Meadow",
        en: Text {
            name: "Workshop",
            summary: "A working session: a short warm-up, one exercise, and what to take away",
            slides: &[
                seed("title", "Workshop: draw the process", &["Ninety minutes, one wall, everyone holds a pen"], ""),
                seed(
                    "titleAndContent",
                    "Plan for the session",
                    &[
                        "Warm-up: five minutes\nExercise: forty minutes in pairs\n\tOne process, drawn start to end\nShare: twenty minutes\nWhat we keep: ten minutes",
                    ],
                    "",
                ),
                seed("sectionHeader", "The exercise", &["Draw how it works today, not how it should"], ""),
                seed(
                    "twoContent",
                    "In pairs",
                    &[
                        "Do\n\tPick one process you know\n\tDraw every step, even the awkward ones",
                        "Don't\n\tFix anything yet\n\tSkip the part that embarrasses you",
                    ],
                    "",
                ),
                seed("titleAndContent", "What to take away", &["One step that surprised you\nOne step nobody owns\nOne thing to try this week"], ""),
                seed("title", "Thank you", &["Photograph the wall before you leave"], ""),
            ],
        },
        es: Text {
            name: "Taller",
            summary: "Una sesión de trabajo: un calentamiento breve, un ejercicio y lo que te llevas",
            slides: &[
                seed("title", "Taller: dibuja el proceso", &["Noventa minutos, una pared, todos con un rotulador"], ""),
                seed(
                    "titleAndContent",
                    "Plan de la sesión",
                    &[
                        "Calentamiento: cinco minutos\nEjercicio: cuarenta minutos por parejas\n\tUn proceso, dibujado de principio a fin\nPuesta en común: veinte minutos\nLo que nos quedamos: diez minutos",
                    ],
                    "",
                ),
                seed("sectionHeader", "El ejercicio", &["Dibuja cómo funciona hoy, no cómo debería"], ""),
                seed(
                    "twoContent",
                    "Por parejas",
                    &[
                        "Sí\n\tElige un proceso que conozcas\n\tDibuja cada paso, también los torpes",
                        "No\n\tArreglar nada todavía\n\tSaltarte la parte que te da vergüenza",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "Lo que te llevas",
                    &["Un paso que te sorprendió\nUn paso que no es de nadie\nUna cosa que probar esta semana"],
                    "",
                ),
                seed("title", "Gracias", &["Fotografía la pared antes de irte"], ""),
            ],
        },
    },
    Starter {
        id: "roadmap",
        theme: "Harbor",
        en: Text {
            name: "Roadmap",
            summary: "Now, next and later, without dates nobody will keep",
            slides: &[
                seed("title", "Roadmap", &["Now, next and later"], ""),
                seed(
                    "titleAndContent",
                    "Now",
                    &[
                        "What we are building this month\n\tThe smallest version that works\nWhat we are fixing first\n\tThe problem people mention most",
                    ],
                    "",
                ),
                seed("titleAndContent", "Next", &["Starts when Now is finished\n\tNothing here has a date\nEach item says what it would change"], ""),
                seed("titleAndContent", "Later", &["Ideas we want to keep, not promise\n\tReviewed every quarter\n\tDropped if nobody asks"], ""),
                seed(
                    "comparison",
                    "Why this order",
                    &[
                        "Now",
                        "People are already waiting for it\nIt is small enough to finish",
                        "Next",
                        "It builds on what we ship now\nIt gets clearer once people use it",
                    ],
                    "",
                ),
                seed("title", "What would you move?", &["Tell us which item should come sooner, and why"], ""),
            ],
        },
        es: Text {
            name: "Hoja de ruta",
            summary: "Ahora, después y más adelante, sin fechas que nadie va a cumplir",
            slides: &[
                seed("title", "Hoja de ruta", &["Ahora, después y más adelante"], ""),
                seed(
                    "titleAndContent",
                    "Ahora",
                    &[
                        "Lo que construimos este mes\n\tLa versión más pequeña que funciona\nLo que arreglamos primero\n\tEl problema que más se menciona",
                    ],
                    "",
                ),
                seed("titleAndContent", "Después", &["Empieza cuando termine Ahora\n\tNada de aquí tiene fecha\nCada punto dice qué cambiaría"], ""),
                seed(
                    "titleAndContent",
                    "Más adelante",
                    &["Ideas que queremos guardar, no prometer\n\tSe revisan cada trimestre\n\tSe descartan si nadie las pide"],
                    "",
                ),
                seed(
                    "comparison",
                    "Por qué este orden",
                    &[
                        "Ahora",
                        "Ya hay gente esperándolo\nEs lo bastante pequeño para terminarlo",
                        "Después",
                        "Se apoya en lo que entregamos ahora\nSe aclara cuando la gente lo usa",
                    ],
                    "",
                ),
                seed("title", "¿Qué moverías?", &["Dinos qué punto debería llegar antes, y por qué"], ""),
            ],
        },
    },
    Starter {
        id: "case study",
        theme: "Ember",
        en: Text {
            name: "Case Study",
            summary: "One piece of work, told as the problem, the choice and the result",
            slides: &[
                seed("title", "Case study: the corner library", &["A small project, told in four steps"], ""),
                seed(
                    "titleAndContent",
                    "The problem",
                    &["The shelves were full and nobody could find anything\n\tFour hundred books, no order\nPeople stopped borrowing"],
                    "",
                ),
                seed("sectionHeader", "What we tried", &["Two ideas, one afternoon each"], ""),
                seed(
                    "comparison",
                    "The choice",
                    &[
                        "Sort by author",
                        "Familiar to everyone\nBreaks when a book has two",
                        "Sort by topic",
                        "Slower to set up\nPeople find things by asking less",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "The result",
                    &["Loans doubled in two months\nThe shelf labels were rewritten by the neighbours\n\tNot by us"],
                    "",
                ),
                seed("title", "What we would do again", &["Ask the readers first. Sort second."], ""),
            ],
        },
        es: Text {
            name: "Caso práctico",
            summary: "Un trabajo contado como problema, decisión y resultado",
            slides: &[
                seed("title", "Caso práctico: la biblioteca de la esquina", &["Un proyecto pequeño, contado en cuatro pasos"], ""),
                seed(
                    "titleAndContent",
                    "El problema",
                    &["Las estanterías estaban llenas y nadie encontraba nada\n\tCuatrocientos libros, sin orden\nLa gente dejó de pedir prestado"],
                    "",
                ),
                seed("sectionHeader", "Lo que probamos", &["Dos ideas, una tarde para cada una"], ""),
                seed(
                    "comparison",
                    "La decisión",
                    &[
                        "Ordenar por autor",
                        "Conocido por todos\nFalla cuando un libro tiene dos",
                        "Ordenar por tema",
                        "Más lento de montar\nLa gente encuentra las cosas preguntando menos",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "El resultado",
                    &["Los préstamos se duplicaron en dos meses\nLas etiquetas de las estanterías las reescribieron los vecinos\n\tNo nosotros"],
                    "",
                ),
                seed("title", "Lo que repetiríamos", &["Preguntar primero a los lectores. Ordenar después."], ""),
            ],
        },
    },
    Starter {
        id: "research talk",
        theme: "Paper",
        en: Text {
            name: "Research Talk",
            summary: "A twenty-minute talk about one question and what the data says",
            slides: &[
                seed("title", "Do short breaks help?", &["A twenty-minute talk about one question"], ""),
                seed(
                    "titleAndContent",
                    "The question",
                    &["Does a five-minute break improve the next hour?\n\tMeasured by errors, not by mood\nWho we asked\n\tForty people, two weeks"],
                    "",
                ),
                seed("sectionHeader", "Method", &["Enough detail that someone could repeat it"], ""),
                seed(
                    "titleAndContent",
                    "What we found",
                    &[
                        "Fewer errors after the break\n\tThe gap was small but steady\nNo difference for short tasks\nOne group did worse, and we do not know why",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "What this does not say",
                    &["Nothing about long-term health\nNothing about other kinds of work\nNothing about breaks longer than ten minutes"],
                    "",
                ),
                seed("title", "Questions", &["Including the ones about what we got wrong"], ""),
            ],
        },
        es: Text {
            name: "Charla de investigación",
            summary: "Una charla de veinte minutos sobre una pregunta y lo que dicen los datos",
            slides: &[
                seed("title", "¿Ayudan las pausas cortas?", &["Una charla de veinte minutos sobre una pregunta"], ""),
                seed(
                    "titleAndContent",
                    "La pregunta",
                    &[
                        "¿Una pausa de cinco minutos mejora la hora siguiente?\n\tMedido en errores, no en ánimo\nA quién preguntamos\n\tCuarenta personas, dos semanas",
                    ],
                    "",
                ),
                seed("sectionHeader", "Método", &["Con el detalle suficiente para que otra persona lo repita"], ""),
                seed(
                    "titleAndContent",
                    "Lo que encontramos",
                    &[
                        "Menos errores después de la pausa\n\tLa diferencia fue pequeña pero constante\nSin diferencia en tareas cortas\nUn grupo lo hizo peor y no sabemos por qué",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "Lo que esto no dice",
                    &["Nada sobre la salud a largo plazo\nNada sobre otros tipos de trabajo\nNada sobre pausas de más de diez minutos"],
                    "",
                ),
                seed("title", "Preguntas", &["También las que tratan de lo que hicimos mal"], ""),
            ],
        },
    },
    Starter {
        id: "welcome",
        theme: "Coral Reef",
        en: Text {
            name: "Welcome",
            summary: "A first-day welcome that says where things are and who to ask",
            slides: &[
                seed("title", "Welcome", &["Your first day, in five slides"], ""),
                seed(
                    "titleAndContent",
                    "Today",
                    &[
                        "Meet the people you will work with\n\tOver coffee, not in a meeting\nGet your accounts and your desk\nRead one document, not twenty",
                    ],
                    "",
                ),
                seed("sectionHeader", "Where things are", &["The three places you will use most"], ""),
                seed(
                    "twoContent",
                    "Who to ask",
                    &[
                        "For work questions\n\tYour teammate, first\n\tThen the channel for your team",
                        "For everything else\n\tThe person who greeted you\n\tThey will know, or find out",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "By Friday",
                    &["You have shipped one small thing\nYou know who owns what\nYou have told us what confused you"],
                    "",
                ),
                seed("title", "Glad you're here", &["Ask early. Nobody remembers not knowing."], ""),
            ],
        },
        es: Text {
            name: "Bienvenida",
            summary: "Una bienvenida para el primer día que dice dónde está cada cosa y a quién preguntar",
            slides: &[
                seed("title", "Bienvenida", &["Tu primer día, en cinco diapositivas"], ""),
                seed(
                    "titleAndContent",
                    "Hoy",
                    &[
                        "Conoce a las personas con las que vas a trabajar\n\tTomando un café, no en una reunión\nRecibe tus cuentas y tu mesa\nLee un documento, no veinte",
                    ],
                    "",
                ),
                seed("sectionHeader", "Dónde está cada cosa", &["Los tres sitios que más vas a usar"], ""),
                seed(
                    "twoContent",
                    "A quién preguntar",
                    &[
                        "Dudas de trabajo\n\tTu compañero, primero\n\tDespués el canal de tu equipo",
                        "Para todo lo demás\n\tLa persona que te recibió\n\tLo sabrá o lo averiguará",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "Para el viernes",
                    &["Has entregado una cosa pequeña\nSabes de quién es cada cosa\nNos has contado qué te confundió"],
                    "",
                ),
                seed("title", "Qué alegría tenerte aquí", &["Pregunta pronto. Nadie recuerda no saber."], ""),
            ],
        },
    },
    Starter {
        id: "retrospective",
        theme: "Slate",
        en: Text {
            name: "Retrospective",
            summary: "A team look-back: what to keep, what to change, and one experiment to try next",
            slides: &[
                seed(
                    "title",
                    "Retrospective",
                    &["What worked, what didn't, and what we try next"],
                    "Say the rule out loud: we talk about the work, not about people.",
                ),
                seed("sectionHeader", "Looking back", &["Facts first, opinions second"], ""),
                seed(
                    "comparison",
                    "Keep and change",
                    &[
                        "Keep",
                        "The habit that made the work easier\nThe meeting that was worth everyone's time",
                        "Change",
                        "The step everyone quietly skips\nThe handoff that gets lost",
                    ],
                    "Let each person add one card before anyone discusses any.",
                ),
                seed(
                    "titleAndContent",
                    "One experiment",
                    &["Pick one change, not five\n\tSay how we will know it worked\nGive it an owner and a date\n\tLook at it again next time"],
                    "",
                ),
                seed("title", "Thank you", &["Small changes that stick beat big ones we forget"], ""),
            ],
        },
        es: Text {
            name: "Retrospectiva",
            summary: "Una mirada atrás del equipo: qué conservar, qué cambiar y un experimento para probar",
            slides: &[
                seed(
                    "title",
                    "Retrospectiva",
                    &["Qué funcionó, qué no y qué probamos ahora"],
                    "Di la regla en voz alta: hablamos del trabajo, no de las personas.",
                ),
                seed("sectionHeader", "Mirando atrás", &["Primero los hechos, después las opiniones"], ""),
                seed(
                    "comparison",
                    "Conservar y cambiar",
                    &[
                        "Conservar",
                        "El hábito que hizo el trabajo más fácil\nLa reunión que mereció el tiempo de todos",
                        "Cambiar",
                        "El paso que todos se saltan sin decirlo\nEl traspaso que se pierde",
                    ],
                    "Deja que cada persona añada una nota antes de comentar ninguna.",
                ),
                seed(
                    "titleAndContent",
                    "Un experimento",
                    &["Elige un cambio, no cinco\n\tDi cómo sabremos que funcionó\nDale responsable y fecha\n\tVolvemos a mirarlo la próxima vez"],
                    "",
                ),
                seed("title", "Gracias", &["Los cambios pequeños que se quedan valen más que los grandes que olvidamos"], ""),
            ],
        },
    },
    Starter {
        id: "year in review",
        theme: "Harbor",
        en: Text {
            name: "Year in Review",
            summary: "A calm look back at the year: highlights, three numbers, and what comes next",
            slides: &[
                seed(
                    "title",
                    "The year in review",
                    &["Where we started, what we learned, where we go next"],
                    "Start with one sentence about how the year felt.",
                ),
                seed(
                    "titleAndContent",
                    "Highlights",
                    &[
                        "What we are proudest of\n\tWhy it mattered to the people it was for\nWhat surprised us\n\tAnd what we did about it\nWhat we would redo\n\tAnd what we would change",
                    ],
                    "",
                ),
                seed("sectionHeader", "By the numbers", &["Three numbers are enough"], "Name each number, say what it measures, and move on."),
                seed(
                    "twoContent",
                    "What went well, what was hard",
                    &[
                        "Went well\n\tThe work that found its people\n\tThe habits that held",
                        "Was hard\n\tThe weeks that ran long\n\tThe things we learned late",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "Next year",
                    &["One thing to start\nOne thing to stop\nOne thing to keep doing, exactly as we do it now"],
                    "",
                ),
                seed("title", "Thank you", &["For the year, and for the next one"], ""),
            ],
        },
        es: Text {
            name: "Balance del año",
            summary: "Una mirada tranquila al año: lo más destacado, tres números y lo que viene",
            slides: &[
                seed(
                    "title",
                    "El año en revisión",
                    &["De dónde partimos, qué aprendimos y adónde vamos"],
                    "Empieza con una frase sobre cómo se sintió el año.",
                ),
                seed(
                    "titleAndContent",
                    "Lo más destacado",
                    &[
                        "De lo que más orgullo sentimos\n\tPor qué importó a quienes iba dirigido\nLo que nos sorprendió\n\tY lo que hicimos al respecto\nLo que repetiríamos\n\tY lo que cambiaríamos",
                    ],
                    "",
                ),
                seed("sectionHeader", "En números", &["Con tres números basta"], "Nombra cada número, di qué mide y sigue."),
                seed(
                    "twoContent",
                    "Lo que fue bien, lo que costó",
                    &[
                        "Fue bien\n\tEl trabajo que encontró a su gente\n\tLos hábitos que se mantuvieron",
                        "Costó\n\tLas semanas que se alargaron\n\tLo que aprendimos tarde",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "El año que viene",
                    &["Una cosa que empezar\nUna cosa que dejar\nUna cosa que seguir haciendo exactamente igual"],
                    "",
                ),
                seed("title", "Gracias", &["Por este año y por el siguiente"], ""),
            ],
        },
    },
    Starter {
        id: "decision brief",
        theme: "Paper",
        en: Text {
            name: "Decision Brief",
            summary: "The choice, two options side by side, and a recommendation people can say yes or no to",
            slides: &[
                seed(
                    "title",
                    "Decision brief",
                    &["The choice, the options and what we recommend"],
                    "State the decision in one sentence before anything else.",
                ),
                seed(
                    "titleAndContent",
                    "The decision",
                    &["What we are deciding\n\tIn one sentence\nWhy now\n\tAnd what changes if we wait\nWho decides\n\tAnd by when"],
                    "",
                ),
                seed(
                    "comparison",
                    "Two options",
                    &[
                        "Option A",
                        "What it costs\nWhat it gives us\nWhat could go wrong",
                        "Option B",
                        "What it costs\nWhat it gives us\nWhat could go wrong",
                    ],
                    "Keep both options the same length: the shape of the page should not argue for one.",
                ),
                seed(
                    "titleAndContent",
                    "Recommendation",
                    &["We recommend one option, and why\n\tThe one reason that settles it\nWhat we do first\n\tAnd how we will know it was right"],
                    "",
                ),
                seed("title", "Decide by Friday", &["A yes, a no, or the one question that is still open"], ""),
            ],
        },
        es: Text {
            name: "Nota de decisión",
            summary: "La decisión, dos opciones lado a lado y una recomendación a la que se pueda decir sí o no",
            slides: &[
                seed(
                    "title",
                    "Nota de decisión",
                    &["La decisión, las opciones y lo que recomendamos"],
                    "Enuncia la decisión en una frase antes que nada.",
                ),
                seed(
                    "titleAndContent",
                    "La decisión",
                    &["Qué estamos decidiendo\n\tEn una sola frase\nPor qué ahora\n\tY qué cambia si esperamos\nQuién decide\n\tY para cuándo"],
                    "",
                ),
                seed(
                    "comparison",
                    "Dos opciones",
                    &[
                        "Opción A",
                        "Lo que cuesta\nLo que nos da\nLo que podría salir mal",
                        "Opción B",
                        "Lo que cuesta\nLo que nos da\nLo que podría salir mal",
                    ],
                    "Mantén las dos opciones igual de largas: la forma de la página no debe inclinar la balanza.",
                ),
                seed(
                    "titleAndContent",
                    "Recomendación",
                    &["Recomendamos una opción, y por qué\n\tLa razón que lo decide\nQué hacemos primero\n\tY cómo sabremos que acertamos"],
                    "",
                ),
                seed("title", "Decidir el viernes", &["Un sí, un no, o la pregunta que sigue abierta"], ""),
            ],
        },
    },
    Starter {
        id: "how-to guide",
        theme: "Meadow",
        en: Text {
            name: "How-to Guide",
            summary: "Four short steps, what you need beforehand, and what to do when it goes wrong",
            slides: &[
                seed("title", "How to set up your workspace", &["Four steps, about ten minutes"], ""),
                seed(
                    "titleAndContent",
                    "Before you start",
                    &["You will need\n\tAn account\n\tTen quiet minutes\nYou will end with\n\tA workspace you can share"],
                    "",
                ),
                seed("sectionHeader", "The four steps", &["One thing at a time, in this order"], ""),
                seed(
                    "twoContent",
                    "Steps one and two",
                    &[
                        "1. Create it\n\tGive it a name people will recognise\n\tPick where it lives",
                        "2. Invite people\n\tStart with the two you work with most\n\tAdd the rest later",
                    ],
                    "",
                ),
                seed(
                    "twoContent",
                    "Steps three and four",
                    &[
                        "3. Set it up\n\tChoose what you see first\n\tTurn off what you don't need",
                        "4. Try it\n\tDo one real task in it\n\tNote what felt slow",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "If something goes wrong",
                    &[
                        "It will not let you in\n\tCheck the address, then ask for a new link\nA person is missing\n\tSend the invitation again\nSomething looks wrong\n\tUndo once and look again",
                    ],
                    "",
                ),
                seed("title", "You are done", &["Ask early. Everyone gets stuck on step three."], ""),
            ],
        },
        es: Text {
            name: "Guía paso a paso",
            summary: "Cuatro pasos breves, lo que necesitas antes y qué hacer cuando algo sale mal",
            slides: &[
                seed("title", "Cómo preparar tu espacio de trabajo", &["Cuatro pasos, unos diez minutos"], ""),
                seed(
                    "titleAndContent",
                    "Antes de empezar",
                    &["Vas a necesitar\n\tUna cuenta\n\tDiez minutos de calma\nAl final tendrás\n\tUn espacio de trabajo que puedes compartir"],
                    "",
                ),
                seed("sectionHeader", "Los cuatro pasos", &["Una cosa cada vez, en este orden"], ""),
                seed(
                    "twoContent",
                    "Pasos uno y dos",
                    &[
                        "1. Créalo\n\tPonle un nombre que la gente reconozca\n\tElige dónde vive",
                        "2. Invita a gente\n\tEmpieza por las dos personas con las que más trabajas\n\tAñade al resto después",
                    ],
                    "",
                ),
                seed(
                    "twoContent",
                    "Pasos tres y cuatro",
                    &[
                        "3. Ajústalo\n\tElige qué ves primero\n\tApaga lo que no necesitas",
                        "4. Pruébalo\n\tHaz en él una tarea de verdad\n\tAnota lo que se sintió lento",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "Si algo sale mal",
                    &[
                        "No te deja entrar\n\tComprueba la dirección y pide un enlace nuevo\nFalta una persona\n\tVuelve a enviar la invitación\nAlgo se ve raro\n\tDeshaz una vez y vuelve a mirar",
                    ],
                    "",
                ),
                seed("title", "Listo", &["Pregunta pronto. Todo el mundo se atasca en el paso tres."], ""),
            ],
        },
    },
    Starter {
        id: "portfolio",
        theme: "Slate",
        en: Text {
            name: "Portfolio",
            summary: "A short introduction to you, two pieces of work, and how you like to work",
            slides: &[
                seed("title", "Hello, I'm Alex", &["Designer, maker, fond of quiet interfaces"], "Replace the name and one line with your own."),
                seed(
                    "titleAndContent",
                    "About",
                    &[
                        "What I do\n\tI make things people use every day\nWhat I care about\n\tClear words and calm screens\nWhere I have worked\n\tA studio, a school and a small shop",
                    ],
                    "",
                ),
                seed("sectionHeader", "Selected work", &["Two pieces I would show first"], ""),
                seed(
                    "twoContent",
                    "Two projects",
                    &[
                        "A booking tool\n\tThe problem it solved\n\tWhat I made\n\tWhat changed afterwards",
                        "A neighbourhood guide\n\tThe problem it solved\n\tWhat I made\n\tWhat changed afterwards",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "How I work",
                    &[
                        "Listen first\n\tTalk to the people who will use it\nMake it small\n\tShow something real early\nFinish properly\n\tThe last ten percent is the work",
                    ],
                    "",
                ),
                seed("title", "Let's talk", &["hello@example.com"], ""),
            ],
        },
        es: Text {
            name: "Portafolio",
            summary: "Una presentación breve de ti, dos trabajos y cómo te gusta trabajar",
            slides: &[
                seed(
                    "title",
                    "Hola, soy Alex",
                    &["Diseñador, hacedor, amigo de las interfaces tranquilas"],
                    "Cambia el nombre y la línea por los tuyos.",
                ),
                seed(
                    "titleAndContent",
                    "Sobre mí",
                    &[
                        "A qué me dedico\n\tHago cosas que la gente usa cada día\nQué me importa\n\tLas palabras claras y las pantallas serenas\nDónde he trabajado\n\tUn estudio, una escuela y una tienda pequeña",
                    ],
                    "",
                ),
                seed("sectionHeader", "Trabajos escogidos", &["Dos que enseñaría primero"], ""),
                seed(
                    "twoContent",
                    "Dos proyectos",
                    &[
                        "Una herramienta de reservas\n\tEl problema que resolvía\n\tLo que hice\n\tQué cambió después",
                        "Una guía del barrio\n\tEl problema que resolvía\n\tLo que hice\n\tQué cambió después",
                    ],
                    "",
                ),
                seed(
                    "titleAndContent",
                    "Cómo trabajo",
                    &[
                        "Escuchar primero\n\tHablar con quienes lo van a usar\nHacerlo pequeño\n\tEnseñar algo real pronto\nTerminarlo bien\n\tEl último diez por ciento es el trabajo",
                    ],
                    "",
                ),
                seed("title", "Hablemos", &["hola@ejemplo.com"], ""),
            ],
        },
    },
    Starter {
        id: "launch plan",
        theme: "Coral Reef",
        en: Text {
            name: "Launch Plan",
            summary: "Who does what and when for a calm release, and the answers to what could go wrong",
            slides: &[
                seed(
                    "title",
                    "Launch plan",
                    &["Who does what, and when, for a calm release"],
                    "Share the date first. Everything else hangs from it.",
                ),
                seed(
                    "titleAndContent",
                    "The goal",
                    &[
                        "What launching means\n\tIn one sentence\nWho it is for\n\tAnd what they can do on day one\nHow we will know it went well\n\tOne number, one week after",
                    ],
                    "",
                ),
                seed("sectionHeader", "The timeline", &["Work backwards from the day"], ""),
                seed(
                    "twoContent",
                    "Before and after launch",
                    &[
                        "Before\n\tFreeze what ships\n\tTell the people who will answer questions\n\tRehearse once",
                        "After\n\tWatch the first day closely\n\tAnswer every question within a day\n\tWrite down what we would change",
                    ],
                    "",
                ),
                seed(
                    "comparison",
                    "Risks and answers",
                    &[
                        "If this happens",
                        "People can't find it\nIt is slower than we said",
                        "We do this",
                        "Point them to one clear page\nSay so, and give the real number",
                    ],
                    "",
                ),
                seed("title", "Ready", &["One date, one owner for each step"], ""),
            ],
        },
        es: Text {
            name: "Plan de lanzamiento",
            summary: "Quién hace qué y cuándo para un lanzamiento sereno, y qué hacer si algo sale mal",
            slides: &[
                seed(
                    "title",
                    "Plan de lanzamiento",
                    &["Quién hace qué, y cuándo, para un lanzamiento sereno"],
                    "Comparte primero la fecha. Todo lo demás cuelga de ella.",
                ),
                seed(
                    "titleAndContent",
                    "El objetivo",
                    &[
                        "Qué significa lanzar\n\tEn una sola frase\nPara quién es\n\tY qué podrá hacer el primer día\nCómo sabremos que fue bien\n\tUn número, una semana después",
                    ],
                    "",
                ),
                seed("sectionHeader", "El calendario", &["Trabaja hacia atrás desde el día"], ""),
                seed(
                    "twoContent",
                    "Antes y después del lanzamiento",
                    &[
                        "Antes\n\tCongelar lo que se entrega\n\tAvisar a quienes responderán dudas\n\tEnsayar una vez",
                        "Después\n\tVigilar de cerca el primer día\n\tResponder cada duda en un día\n\tAnotar lo que cambiaríamos",
                    ],
                    "",
                ),
                seed(
                    "comparison",
                    "Riesgos y respuestas",
                    &[
                        "Si ocurre esto",
                        "La gente no lo encuentra\nVa más lento de lo que dijimos",
                        "Hacemos esto",
                        "Llevarles a una página clara\nDecirlo y dar la cifra real",
                    ],
                    "",
                ),
                seed("title", "Listos", &["Una fecha, un responsable para cada paso"], ""),
            ],
        },
    },
];

/// The starter gallery, in the order File › New shows it.
pub fn starters() -> &'static [Starter] {
    STARTERS
}

/// Starters whose name, summary, theme or a slide title in `lang` contains `query` (empty matches all).
pub fn matching(query: &str, lang: Lang) -> Vec<&'static Starter> {
    let q = query.trim().to_lowercase();
    let hit = |text: &str| text.to_lowercase().contains(&q);
    starters()
        .iter()
        .filter(|t| {
            let text = t.text(lang);
            q.is_empty() || hit(text.name) || hit(text.summary) || hit(t.theme) || text.slides.iter().any(|slide| hit(slide.title))
        })
        .collect()
}

/// A starter by id or by its name in either language, ignoring case.
pub fn find(name: &str) -> Option<&'static Starter> {
    let name = name.trim();
    starters().iter().find(|t| t.id.eq_ignore_ascii_case(name) || t.en.name.eq_ignore_ascii_case(name) || t.es.name.eq_ignore_ascii_case(name))
}

fn text_with_levels(text: &str) -> TextBody {
    let mut body = TextBody::from_text(text);
    for para in &mut body.paragraphs {
        let tabs = para.runs.first().map(|r| r.text.chars().take_while(|c| *c == '\t').count()).unwrap_or(0);
        if tabs > 0
            && let Some(run) = para.runs.first_mut()
        {
            run.text = run.text.trim_start_matches('\t').to_string();
            para.level = tabs.min(8) as u8;
        }
    }
    body
}

/// A new presentation filled with `name`'s slides, written in `lang`. `size` is the slide size in points.
pub fn build(name: &str, size: Size, lang: Lang) -> Result<Presentation> {
    let starter = find(name).ok_or_else(|| {
        let names = starters().iter().map(|t| t.en.name).collect::<Vec<_>>().join(", ");
        bad("file.new", format!("no template `{name}`; use {names}"))
    })?;
    let text = starter.text(lang);
    let theme = lamina_model::theme::builtin_themes()
        .into_iter()
        .find(|t| t.name.eq_ignore_ascii_case(starter.theme))
        .ok_or_else(|| bad("file.new", format!("template `{}` names a missing theme `{}`", starter.id, starter.theme)))?;
    let mut doc = defaults::blank_presentation(size, theme, false);
    for seed in text.slides {
        let key = Value::String(seed.layout.to_string());
        let layout = find_layout(&doc, &key, 0)
            .ok_or_else(|| bad("file.new", format!("template `{}` uses an unknown layout `{}`", starter.id, seed.layout)))?;
        let mut slide = defaults::new_slide(&mut doc, layout).ok_or_else(|| bad("file.new", crate::cmd::NO_ID_SPACE))?;
        if !seed.title.is_empty()
            && let Some(sh) = slide.shapes.iter_mut().find(|sh| sh.ph_type().is_some_and(|k| k.is_title()))
        {
            sh.text = Some(text_with_levels(seed.title));
        }
        let mut bodies = seed.bodies.iter();
        for sh in slide.shapes.iter_mut().filter(|sh| sh.ph_type().is_some_and(|k| !k.is_title())) {
            let Some(body) = bodies.next() else { break };
            if !body.is_empty() {
                sh.text = Some(text_with_levels(body));
            }
        }
        if bodies.next().is_some() {
            return Err(bad("file.new", format!("template `{}` has more text than the `{}` layout holds", starter.id, seed.layout)));
        }
        if !seed.notes.is_empty() {
            slide.notes = text_with_levels(seed.notes);
        }
        doc.slides.push(Arc::new(slide));
    }
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::Session;

    fn body(doc: &Presentation, slide: usize, n: usize) -> (String, u8) {
        let para_slide = &doc.slides[slide];
        let text = para_slide
            .shapes
            .iter()
            .filter(|sh| sh.ph_type().is_some_and(|k| !k.is_title()))
            .nth(n)
            .and_then(|sh| sh.text.as_ref())
            .cloned()
            .unwrap_or_default();
        let level = text.paragraphs.get(1).map(|p| p.level).unwrap_or(0);
        (text.text(), level)
    }

    #[test]
    fn every_starter_opens_on_its_theme_and_is_a_clean_document() {
        let mut s = Session::new();
        for starter in starters() {
            s.execute("file.new", &json!({"template": starter.id})).unwrap();
            let st = s.active().unwrap();
            assert_eq!(st.doc.masters[0].theme.name, starter.theme, "{}", starter.id);
            assert_eq!(st.doc.slides.len(), starter.en.slides.len(), "{}", starter.id);
            assert_eq!(st.doc.slides[0].title(), starter.en.slides[0].title, "{}", starter.id);
            assert!(!st.is_dirty(), "{}", starter.id);
            assert!(st.history.undo.is_empty(), "{}", starter.id);
            assert_eq!(st.selection.slide, 0);
        }
        let listed = s.execute("file.templates", &json!({})).unwrap();
        assert_eq!(listed["templates"].as_array().map(Vec::len), Some(starters().len()));
    }

    #[test]
    fn every_starter_is_written_in_both_languages_with_the_same_shape() {
        let mut ids = std::collections::HashSet::new();
        for starter in starters() {
            assert!(ids.insert(starter.id), "duplicate id {}", starter.id);
            assert_eq!(starter.en.slides.len(), starter.es.slides.len(), "{}", starter.id);
            for (en, es) in starter.en.slides.iter().zip(starter.es.slides) {
                assert_eq!(en.layout, es.layout, "{}", starter.id);
                assert_eq!(en.bodies.len(), es.bodies.len(), "{}: {}", starter.id, en.title);
                assert_eq!(en.notes.is_empty(), es.notes.is_empty(), "{}: {}", starter.id, en.title);
                for (a, b) in en.bodies.iter().zip(es.bodies) {
                    assert_eq!(a.lines().count(), b.lines().count(), "{}: {}", starter.id, en.title);
                }
            }
            assert_ne!(starter.en.name, starter.es.name, "{} has no Spanish name", starter.id);
            assert_ne!(starter.en.slides[0].title, starter.es.slides[0].title, "{}", starter.id);
            let build_both = [Lang::En, Lang::Es].map(|l| build(starter.id, defaults::WIDE, l));
            assert!(build_both.iter().all(Result::is_ok), "{}", starter.id);
        }
    }

    #[test]
    fn a_spanish_interface_gets_a_spanish_deck() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"template": "briefing", "lang": "es-ES"})).unwrap();
        assert_eq!(s.active().unwrap().doc.slides[0].title(), "Resumen semanal");
        s.execute("file.new", &json!({"template": "Resumen semanal"})).unwrap();
        assert_eq!(s.active().unwrap().doc.slides[0].title(), "Weekly briefing");
        let listed = s.execute("file.templates", &json!({"lang": "es"})).unwrap();
        assert_eq!(listed["templates"][0]["name"], "Resumen semanal");
        assert_eq!(listed["templates"][0]["id"], "briefing");
        assert_eq!(Lang::parse("es_MX.UTF-8"), Lang::Es);
        assert_eq!(Lang::parse("fr"), Lang::En);
    }

    #[test]
    fn briefing_keeps_notes_columns_and_outline_levels() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"template": "Briefing", "size": [800.0, 450.0]})).unwrap();
        let doc = &s.active().unwrap().doc;
        assert_eq!(doc.slide_size.width, 800.0);
        assert_eq!(doc.slides[0].notes_text(), "Open with what changed since last week.");
        let (agenda, level) = body(doc, 1, 0);
        assert!(agenda.contains("One concrete change"), "{agenda}");
        assert_eq!(level, 1);
        let (done, _) = body(doc, 3, 0);
        let (next, _) = body(doc, 3, 1);
        assert!(done.starts_with("Done"), "{done}");
        assert!(next.starts_with("Next"), "{next}");
    }

    #[test]
    fn pitch_comparison_fills_both_columns() {
        let doc = build("Pitch", defaults::WIDE, Lang::En).unwrap();
        assert_eq!(body(&doc, 3, 0).0, "Today");
        assert!(body(&doc, 3, 1).0.contains("buried"));
        assert_eq!(body(&doc, 3, 2).0, "After");
        assert!(body(&doc, 3, 3).0.contains("top"));
    }

    #[test]
    fn search_matches_name_summary_and_theme_in_the_language_shown() {
        let names = |q: &str, lang| matching(q, lang).iter().map(|t| t.id).collect::<Vec<_>>();
        assert_eq!(names("ridge", Lang::En), vec!["field notes"]);
        assert_eq!(names("nocturne", Lang::En), vec!["evening talk"]);
        assert_eq!(names("cresta", Lang::Es), vec!["field notes"]);
        assert!(names("cresta", Lang::En).is_empty());
        assert_eq!(names("", Lang::En).len(), starters().len());
        assert!(names("facet", Lang::En).is_empty());
    }

    #[test]
    fn unknown_blank_and_theme_are_refused() {
        let mut s = Session::new();
        let unknown = s.execute("file.new", &json!({"template": "Ion"})).unwrap_err();
        assert!(unknown.to_string().contains("Briefing"), "{unknown}");
        let both = s.execute("file.new", &json!({"template": "Lesson", "theme": "Harbor"})).unwrap_err();
        assert!(both.to_string().contains("not both"), "{both}");
        let blank = s.execute("file.new", &json!({"template": "Lesson", "blank": true})).unwrap_err();
        assert!(blank.to_string().contains("blank"), "{blank}");
        assert!(s.active().is_none());
    }

    #[test]
    fn a_starter_round_trips_through_pptx() {
        for lang in [Lang::En, Lang::Es] {
            let doc = build("Evening Talk", defaults::WIDE, lang).unwrap();
            let bytes = lamina_pptx::export(&doc).unwrap();
            let back = lamina_pptx::import(&bytes).unwrap();
            assert_eq!(back.slides.len(), doc.slides.len());
            for (before, after) in doc.slides.iter().zip(back.slides.iter()) {
                assert_eq!(after.title(), before.title());
            }
        }
        let doc = build("Evening Talk", defaults::WIDE, Lang::En).unwrap();
        let back = lamina_pptx::import(&lamina_pptx::export(&doc).unwrap()).unwrap();
        assert!(back.slides[3].shapes.iter().any(|sh| sh.text.as_ref().is_some_and(|t| t.text().contains("bedroom"))));
    }
}
