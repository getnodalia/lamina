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
