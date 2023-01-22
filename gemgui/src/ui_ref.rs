use std::time::Duration;

use crate::Result;
use crate::GemGuiError;
use crate::JSMessageTx;
use crate::JSType;
use crate::element::Element;
use crate::element::Elements;
use crate::ui::Ui;
use crate::ui::private;
use crate::ui_data::ROOT_ID;
use crate::ui_data::UiDataRef;
use crate::ui_data::UiData;



/// Refence to UI
pub struct UiRef {
    ui: UiDataRef,
}

impl private::UserInterface for UiRef {
    fn ui(&self) -> &UiDataRef {
        &self.ui
    }
}

impl Ui for UiRef {}

/*

UiRef and Ui shares implementation by composition

Trait default implementation cannot be used as that interface trait has to be set public (as it is the actual
API) - however only UiRef and Ui are the API

*/

impl UiRef {

    pub (crate) fn new(ui: UiDataRef) -> UiRef {
        UiRef { ui }
    }

    /// Get UI native pixel ratio
    /// 
    /// # Return
    /// 
    /// ratio
    pub async fn device_pixel_ratio(&self) -> Result<f32> {
        let value = self.query("", "devicePixelRatio", &vec![]).await?;
        if value.is_number() {
            Ok(value.as_f64().unwrap() as f32)
        } else {
            Err(GemGuiError::Err(format!("Not a number {}", value)))
        } 
    }

    /// Whether an control with a name exists
    /// 
    /// # Return
    /// 
    /// boolean
    pub async fn exists(&self, id: &str) -> Result<bool> {
        let value = self.query(id, "exists", &vec!()).await?;
        if value.is_boolean() {
            Ok(value.as_bool().unwrap())
        } else {
            GemGuiError::error(&format!("Not a bool {}", value))
        } 
    }

    /// Get elements by their class
    /// 
    /// # Arguments
    /// 
    /// `class_name` - class name
    /// 
    /// # Return
    /// 
    /// Vector of elements
    pub async fn by_class(&self, class_name: &str) -> Result<Elements> {
        // query parameters are bit odd...
        let result = self.query(class_name, "classes", &vec![]).await;
        match result {
            Ok(value) => UiData::elements_from_values(&self.ui, value, &UiData::sender(&self.ui)),
            Err(e) => Err(e),
        }
    }


    /// Get elements by their name
    /// 
    /// # Arguments
    /// 
    /// `cname` - name
    /// 
    /// # Return
    /// 
    /// Vector of elements
    pub async fn by_name<Str : Into<String>>(&self, name: Str) -> Result<Elements> {
        let class_name = name.into();
        // query parameters are bit odd...
        let result = self.query(&class_name, "names", &vec![]).await;
        match result {
            Ok(value) => UiData::elements_from_values(&self.ui, value, &UiData::sender(&self.ui)),
            Err(e) => Err(e),
        }
    }



    fn contains_id(&self, key: &str) -> bool {
        let ui = self.ui.lock().unwrap();
        ui.elements.contains_key(key)
    }
    

    /// Create a new element
    /// 
    /// # Example
    /// 
    /// ```
    /// # use gemgui::ui_ref::UiRef;
    /// # use crate::gemgui::ui::Ui;
    /// async fn some_function(ui: UiRef) {
    ///     let el = ui.add_element("div", &ui.root()).await.unwrap();
    ///     el.set_html("foo");
    /// }
    /// ```
    ///  # Arguments
    /// 
    /// `html_element` - refer to HTML id
    /// 
    /// `parent` - parent of element
    pub async fn add_element(&self, html_element: &str, parent: &Element) -> Result<Element> { 
        self.add_element_with_id(&UiData::random_element_id(&self.ui), html_element, parent).await
    }

    /// Create a new element
    /// 
    ///  See more information [add_element](UiRef::add_element)
    /// 
    ///  # Arguments
    /// 
    /// `id` - id of element - is expexted to be unique in the application context
    /// 
    /// `html_element` - refer to HTML id
    /// 
    /// `parent` - parent of element
    /// 
    /// 
    pub async fn add_element_with_id(&self, id: &str, html_element: &str, parent: &Element) -> Result<Element> { 

        if id == ROOT_ID || self.contains_id(id)/*  ||  self.exists(id).await? */{
            return GemGuiError::error("Bad id");
        } 

        UiData::insert_element(&self.ui, id);

        let ui = self.ui.clone();
        let element = Element::construct(id.to_string(), UiData::sender(&ui), ui);
        element.create(html_element, parent);

        let mut exists = false;

        for _i in 0..10 {
            if self.exists(id).await? {
                exists = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        if !exists {
            return GemGuiError::error("Element state unclear");
        }
        

        Ok(element)



    }
    

    pub (crate) async fn query(&self, target: &str, name: &str, query_params: &Vec<String>) -> Result<JSType> {
        Self::do_query(&self.ui, target, name, query_params).await
    }

    // queries are only applicable on async context, hence only available on UiRef
    pub (crate) async fn do_query(ui: &UiDataRef, target: &str, name: &str, query_params: &Vec<String>) -> Result<JSType> {
        let (id, receiver) = UiData::new_query(ui);
        let msg =  JSMessageTx {
            element: target,
            _type: "query",
            query_id: Some(&id),
            query: Some(name),
            query_params: Some(query_params),
            ..Default::default()
        };

        UiData::send(ui, msg);

        // spawn an syncrnous wait and wait that async
        let value = tokio::task::spawn_blocking(move || {
            receiver.blocking_recv()
        }).await.unwrap_or_else(|e| {panic!("Query spawn blocking {:#?}", e)});

        match value {
            Ok(v) => Ok(v),
            Err(e) => Err(GemGuiError::Err(format!("Query error {}", e)))
        }        
    }

}
