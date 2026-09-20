import { ComponentFixture, TestBed } from '@angular/core/testing';
import { SemanticSearchComponent } from './semantic-search.component';
import { HttpClientTestingModule } from '@angular/common/http/testing';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';

describe('SemanticSearchComponent', () => {
  let component: SemanticSearchComponent;
  let fixture: ComponentFixture<SemanticSearchComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [
        SemanticSearchComponent,
        HttpClientTestingModule,
        BrowserAnimationsModule
      ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(SemanticSearchComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
