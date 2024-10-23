# API-ICT

This API allows you to list all ICT modules

## API Endpoints

### 1. Get Documentation

-   **URL:** `/`
-   **Method:** `GET`
-   **Description:** Retrieve the API documentation for available routes and usage.

### 2. Get List of Job

-   **URL:** `/jobs`
-   **Method:** `GET`
-   **Description:** Retrieve a list of Job with their IDs.
-   **Response:**
    ```json
    [
    	{ "id": 89494, "name": "Job A" },
    	{ "id": 83947, "name": "Job B" }
    ]
    ```

### 3. Get all modules

-   **URL:** `/modules`
-   **Method:** `GET`
-   **Description:** Get a list of modules filtered by _job ID_.
-   **Parameters:**
    -   jobId (optional): Module group ID (e.g., 89494).
    -   lang (optional): Response language (FR, DE, IT).
    -   year (optional): Year of the modules (e.g., 2024).
-   **Response:**
    ```json
    [
    	{ "creation_date": "2021-09-28T05:33:38Z", "description": "Planung der Installation eines neuen lokalen Netzwerks ohne zentrale Benutzerverwaltung mit bis zu 10 Arbeitsplätzen und Internetanschluss, das Computer und Drucker in verschiedenen Räumen des gleichen Gebäudes miteinander verbindet. Installation der Netzwerkkomponenten ab der LAN-Steckdose (Computer, lokaler Drucker).", "last_modified": "2024-07-22T11:32:32Z", "name": "Informatik- und Netzinfrastruktur für ein kleines Unternehmen realisieren", "number": 117, "type": "Berufsfachschule", "version": 4, "year": 1 },
    	{ "creation_date": "2021-02-11T14:03:15Z", "description": "Aufträge im eigenen Berufsumfeld mit definierten Zielen und Ergebnissen.", "last_modified": "2024-07-22T13:29:08Z", "name": "Aufträge im eigenen Berufsumfeld selbstständig durchführen", "number": 431, "type": "Berufsfachschule", "version": 2, "year": 1 }
    ]
    ```

### 4. Get Module Details

-   **URL:** `/modules/{moduleNumber}`
-   **Method:** `GET`
-   **Description:** Retrieve detailed information for a specific module by _Number_.
-   **Parameters:**
    -   lang (optional): Response language (FR, DE, IT).
    -   year (optional): Year of the modules (e.g., 2024).
-   **Response:**
    ```json
    {
    	"competence": "Liest Daten zu Analyse ein, validiert diese und erstellt mit Hilfe geeigneter Metriken, Grenzwerte und Indikatoren Reports und graphische Darstellungen von Monitoring Daten. Gibt Daten anonymisiert an Dritte weiter. ",
    	"creation_date": "2021-02-26T07:44:10Z",
    	"description": "Unbearbeitete Monitoring Daten (z.B. Logdaten)",
    	"last_modified": "2023-02-20T07:14:33Z",
    	"name": "Daten mit Tools analysieren und darstellen",
    	"number": 110,
    	"objectives": [
    		{ "details": ["Kennt Indikatoren, welche für die Bildung von Reports benötigt werden (z.B. Zeit, Fehlerhäufigkeit). 110.1.1", "Kennt den Aufbau eines Reports. 110.1.2"], "name": "Gewinnt aus vorhandenen Daten Indikatoren für Bildung von Reports." },
    		{ "details": ["Kennt die Definition eines Grenzwertes für Alerts. 110.2.1", "Kennt Methoden, um Alerts zu erstellen. 110.2.2"], "name": "Definiert Grenzwerte für Alerts." }
    	],
    	"pdf": "https://www.modulbaukasten.ch/Module/110_1_Daten%20mit%20Tools%20analysieren%20und%20darstellen.pdf",
    	"type": "",
    	"version": 1,
    	"year": 0
    }
    ```

## Deploying the API-ICT

The API-ICT application runs in a Docker container. You can start it using the following command:

```bash
docker run -d -p 8000:8000 --name api-ict dwesh163/api-ict
```

### Optional Parameters

You can customize the deployment with the following environment variables:

-   `DISABLE_CACHE`: Set to `True` to disable caching. (default is `FALSE`)
-   `DEFAULT_LANGUAGE`: Specify the default language (e.g., `FR` for French, default is `DE`).
-   `PORT`: Set the port on which the application will run (default is `8000`).

### Example Command with Optional Parameters

To run the container with optional parameters, use the following command:

```bash
docker run -d -p 8000:8000 --name api-ict \
 -e DEFAULT_LANGUAGE=FR \
 dwesh163/api-ict
```
